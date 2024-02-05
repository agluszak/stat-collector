use crate::logic::email::EmailAttachment;
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub fn reminder(
    reminder_text: PreEscaped<&str>,
    header_img: &EmailAttachment,
    body_img: &EmailAttachment,
    footer_img: &EmailAttachment,
    dont_print_img: &EmailAttachment,
    link: &str,
) -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta http-equiv="Content-Type" content="text/html; charset=utf-8";
            title { "" }
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com";
            link href="https://fonts.googleapis.com/css2?family=League+Spartan:wght@700&display=swap" rel="stylesheet";
            link href="https://fonts.googleapis.com/css2?family=League+Spartan&display=swap" rel="stylesheet";
            style type="text/css" {
                {r#"
                    body {
                        background-color: #fff;
                    }
                    td {
                        mso-table-lspace: 0;
                        mso-table-rspace: 0;
                    }
                    table {
                        border-collapse: collapse;
                    }
                    a:link {
                        text-decoration: none !important;
                    }
                    .league-spartan-bold {
                        font-family: "League Spartan", sans-serif;
                        font-optical-sizing: auto;
                        font-weight: 700;
                        font-style: normal;
                        line-height: 1.4;
                    }
                    .league-spartan-regular {
                        font-family: "League Spartan", sans-serif;
                        font-optical-sizing: auto;
                        font-weight: 400;
                        font-style: normal;
                        line-height: 1.05;
                    }
                    @media only screen and (max-width: 640px) {
                        .img {
                            width: 100% !important;
                            height: auto !important;
                        }
                        .max {
                            width: 100% !important;
                        }
                    }
                "#}
            }
        }

        body {
            table width="100%" border="0" cellspacing="0" cellpadding="0" {
                tbody {
                    tr {
                        td align="center" valign="middle" {
                            table width="600" border="0" align="center" cellpadding="0" cellspacing="0" class="max" {
                                tbody {
                                    tr {
                                        td {
                                            table width="600" border="0" cellspacing="0" cellpadding="0" class="max" {
                                                tbody {
                                                    tr {
                                                        td {
                                                            a href=(link) target="_blank" {
                                                                img style="display: block; border: 0;" border="0" class="img" src=(header_img.as_src()) width="600" height="235" alt="";
                                                            }
                                                        }
                                                    }
                                                    tr height="211px" background=(body_img.as_src()) {
                                                        td style="padding-left: 10%; padding-bottom: 2%;" {
                                                            span class="league-spartan-bold" style="font-size: 23px; line-height: 1.5;" {
                                                                "Cześć,"
                                                                br;
                                                            }
                                                            span class="league-spartan-regular" style="font-size: 23px; line-height: 1.3;" {
                                                                (reminder_text)
                                                            }
                                                        }
                                                    }
                                                    tr {
                                                        td {
                                                            a href=(link) target="_blank" {
                                                                img style="display: block; border: 0;" border="0" class="img" src=(footer_img.as_src()) width="600" height="auto" alt="";
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            table width="600" border="0" cellspacing="0" cellpadding="0" class="max" {
                                                tbody {
                                                    tr {
                                                        td width="6%" {}
                                                        td width="94%" class="league-spartan-bold" style="font-size: 13px; padding-top: 2%;" {
                                                            span {
                                                                "Stats Collector"
                                                                br;
                                                                "SIGMA BIS S.A."
                                                                br;
                                                                "ul. Bielańska 12, 00-085 Warszawa"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            table width="600" border="0" cellspacing="0" cellpadding="0" class="max" {
                                                tbody {
                                                    tr {
                                                        td width="6%" {}
                                                        td width="88%" class="league-spartan-regular" style="font-size: 9px; padding-top: 1%;" {
                                                            span {
                                                                br;
                                                                br;
                                                                "Sigma BIS Spółka Akcyjna z siedzibą w Warszawie, ul. Bielańska 12, 00-085 Warszawa"
                                                                br;
                                                                "wpisaną do rejestru przedsiębiorców Krajowego Rejestru Sądowego, prowadzonego przez Sąd Rejonowym dla m. st. Warszawy w Warszawie, XII Wydział Gospodarczy Krajowego Rejestru Sądowego, pod numerem KRS: 0000534906, o kapitale zakładowym (w pełni opłaconym) w wysokości: 3.100.000,00 zł, nr NIP: 527-273-18-26"
                                                                br;
                                                                "UWAGA: Informacja zawarta w niniejszej wiadomości lub którymkolwiek z jej załączników podlega ochronie i jest objęta zakazem ujawniania. Jeśli czytelnik niniejszej wiadomości nie jest jej zamierzonym adresatem lub pośrednikiem upoważnionym do jej przekazania adresatowi, niniejszym informujemy, że wszelkie ujawnianie, w tym przekazanie osobom trzecim, rozprowadzanie, dystrybucja, powielanie niniejszej wiadomości lub jej załączników, bądź inne działanie o podobnym charakterze jest zabronione. Jeżeli otrzymałeś tę wiadomość omyłkowo, prosimy niezwłocznie zawiadomić nadawcę wysyłając odpowiedź na niniejszą wiadomość, a następnie usunąć ją z komputera bez otwierania załączników. Dziękujemy. Sigma BIS S.A."
                                                                br;
                                                                "Sigma BIS SA, a Polish company, with its registered office at ul. Bielańska 12, 00-085 Warszawa"
                                                                br;
                                                                "entered into the Register of Entrepreneurs kept by the District Court for Warsaw in Warsaw, XII Commercial Division of the National Court Register under the number KRS 0000534906 NIP: 527-273-18-26, share capital/paid up capital: 3.100.000,00 PLN"
                                                                br;
                                                                "NOTE: Information contained in this message or any attachments thereto may be protected and withheld from disclosure. Please be advised that if you are not the intended addressee or an authorised recipient of the addressee, any disclosure of this message, including forwarding it or any attachments thereto to third parties, dissemination, distribution, reproduction or any similar activity is prohibited. If you are not the intended recipient of this message, please promptly notify the sender by replying to this message, and then delete it from your computer without opening the attachments. Thank you Sigma BIS S.A."
                                                            }
                                                        }
                                                        td width="6%" {}
                                                    }
                                                }
                                            }
                                            table width="600" border="0" cellspacing="0" cellpadding="0" class="max" {
                                                tbody {
                                                    tr {
                                                        td width="6%" {}
                                                        td width="94%" {
                                                            br;
                                                            img style="display: block; border: 0;" border="0" class="img" src=(dont_print_img.as_src()) width="210" height="75" alt="Oszczędność mamy w naturze - Sprawdź";
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
