use crate::db::{StatisticsCollector, SupplierId};
use crate::email_templates::reminder;
use crate::errors::AppError;
use derive_more::Display;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Body, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};
use maud::PreEscaped;
use mockall::*;
use serde::{Deserialize, Serialize};

use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Copy, Display, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderType {
    FirstReminder,
    SecondReminder,
}

#[automock]
pub trait Mailer: Send + Sync + 'static {
    fn send_reminder(
        &self,
        stat_collector: StatisticsCollector,
        to_email: Address,
        supplier_id: SupplierId,
        reminder_type: ReminderType,
    ) -> Result<(), AppError>;
}

#[derive(Debug, Clone)]
pub struct AppMailer {
    transport: SmtpTransport,
    from_email: Mailbox,
    base_url: String,
}

impl AppMailer {
    pub fn new(
        from_email: Mailbox,
        smtp_server: &str,
        port: u16,
        timeout: Duration,
        creds: Credentials,
        base_url: &str,
    ) -> Self {
        let transport = SmtpTransport::builder_dangerous(smtp_server)
            .port(port)
            .timeout(Some(timeout))
            .credentials(creds)
            .build();

        Self {
            from_email,
            transport,
            base_url: base_url.to_string(),
        }
    }
}

static BODY_IMG: &[u8] = include_bytes!("../../assets/body.jpg");
static FOOTER_IMG: &[u8] = include_bytes!("../../assets/footer.jpg");
static HEADER_IMG: &[u8] = include_bytes!("../../assets/header.jpg");
static HEADER_EXCLAMATION_IMG: &[u8] = include_bytes!("../../assets/header_exclamation.jpg");
static DONT_PRINT_IMG: &[u8] = include_bytes!("../../assets/dont_print.jpg");

pub struct EmailAttachment {
    name: String,
    data: SinglePart,
}

impl EmailAttachment {
    fn new(name: &str, data: &[u8], mime_type: ContentType) -> Self {
        let data =
            Attachment::new_inline(name.to_string()).body(Body::new(data.to_vec()), mime_type);

        Self {
            name: name.to_string(),
            data,
        }
    }

    pub fn as_src(&self) -> String {
        format!("cid:{}", self.name)
    }

    fn into_single_part(self) -> SinglePart {
        self.data
    }
}

impl Mailer for AppMailer {
    fn send_reminder(
        &self,
        stat_collector: StatisticsCollector,
        to_email: Address,
        supplier_id: SupplierId,
        reminder_type: ReminderType,
    ) -> Result<(), AppError> {
        info!(
            "Sending {} about supplier {} to {}",
            reminder_type, supplier_id, to_email
        );
        let to_email = to_email.into();
        let subject = match reminder_type {
            ReminderType::FirstReminder => format!(
                "Prośba o statystyki do kampanii {} dla klienta {}",
                stat_collector.name, stat_collector.client
            ),
            ReminderType::SecondReminder => format!(
                "Przypomnienie: Prośba o statystyki do kampanii {} dla klienta {}",
                stat_collector.name, stat_collector.client
            ),
        };

        let url = format!("{}/supplier/{}", self.base_url, supplier_id);

        let reminder_text = match reminder_type {
            ReminderType::FirstReminder => PreEscaped(
                "wypełnij proszę pilnie statystyki do kampanii.<br>
                                        Poniżej link do tabelki.<br>
                                        Dzięki!<br>
                                        ",
            ),
            ReminderType::SecondReminder => PreEscaped(
                "widzimy, że jeszcze nie wypełniłeś statystyk.<br>
            Zrób to proszę pilnie, bo potrzebujemy<br>
            tego na teraz! Poniżej link do tabelki.<br>
            Dzięki z góry!
            ",
            ),
        };
        let jpeg: ContentType = "image/jpeg".parse().unwrap();

        let header_image = match reminder_type {
            ReminderType::FirstReminder => HEADER_IMG,
            ReminderType::SecondReminder => HEADER_EXCLAMATION_IMG,
        };

        let header = EmailAttachment::new("header", header_image, jpeg.clone());
        let body = EmailAttachment::new("body", BODY_IMG, jpeg.clone());
        let footer = EmailAttachment::new("footer", FOOTER_IMG, jpeg.clone());
        let dont_print = EmailAttachment::new("dont_print", DONT_PRINT_IMG, jpeg.clone());

        let html = reminder(reminder_text, &header, &body, &footer, &dont_print, &url);

        let body = MultiPart::related()
            .singlepart(SinglePart::html(html.into_string()))
            .singlepart(header.into_single_part())
            .singlepart(body.into_single_part())
            .singlepart(footer.into_single_part())
            .singlepart(dont_print.into_single_part());

        let email = Message::builder()
            .from(self.from_email.clone())
            .reply_to(self.from_email.clone())
            .to(to_email)
            .subject(subject)
            .multipart(body)?;

        self.transport.send(&email)?;

        Ok(())
    }
}
