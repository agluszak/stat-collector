use crate::db::{StatisticsCollector, SupplierId};
use crate::errors::AppError;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Mailer {
    transport: SmtpTransport,
    from_email: Mailbox,
    base_url: String,
}

impl Mailer {
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

        Mailer {
            from_email,
            transport,
            base_url: base_url.to_string(),
        }
    }

    pub fn send(
        &self,
        stat_collector: StatisticsCollector,
        to_email: Address,
        supplier_id: SupplierId,
    ) -> Result<(), AppError> {
        info!("Sending email to {}", to_email);
        let to_email = to_email.into();
        let subject = format!(
            "Statystyki kampanii {} dla klienta {}",
            stat_collector.name, stat_collector.client
        );
        let body = format!(
            "Prosimy o uzupełnienie statystyk\n\n{}/supplier/{}",
            self.base_url, supplier_id
        );

        let email = Message::builder()
            .from(self.from_email.clone())
            .reply_to(self.from_email.clone())
            .to(to_email)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        self.transport.send(&email)?;

        Ok(())
    }
}
