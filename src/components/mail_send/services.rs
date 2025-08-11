use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use actix_web::web;
use crate::components::config::ConfigService;

#[derive(Clone)]
pub struct MailSendService {

}

impl MailSendService {
    pub fn new() -> Self {
       MailSendService{}
    }

    pub fn send_mail(
        &self,
        email: String,
        token: String,
        config_service: &ConfigService,
    ) -> Result<(), lettre::transport::smtp::Error> {
        // Construct the full verification URL using the provided token.
        let verification_link = format!("{}/v1/auth/verify/{}", config_service.port_host, token);

        // Build the email message dynamically.
        // The recipient is now the 'email' parameter.
        let email_message = Message::builder()
            .from(Mailbox::new(
                Option::from("Verified email no replay".to_owned()),
                config_service.email_address.parse().unwrap(),
            ))
            // Use the provided `email` parameter for the recipient
            .to("nechiforelsamuel@gmail.com".parse().unwrap())
            .subject("Verify your email")
            .header(ContentType::TEXT_PLAIN)
            // The body now includes the dynamic verification link.
            .body(format!(
                "Please click the following link to verify your email: {}",
                verification_link
            ))
            .unwrap();

        let creds = Credentials::new(config_service.email_address.to_owned(), config_service.smtp_password.to_owned());

        let mailer = SmtpTransport::relay(config_service.smtp_transport.as_str())
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email and return the result.
        mailer.send(&email_message).map(|_| ())
    }
}
