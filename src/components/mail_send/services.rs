use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
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

        // Determine recipient based on environment
        let recipient_email = if config_service.app_env.trim().to_lowercase() == "production" {
            email  // Use actual user email in production
        } else {
            "nechiforelsamuel@gmail.com".to_string()  // Safe default for dev/test
        };

        println!("[MailSend] Environment: {} | Sending to: {}",
            config_service.app_env, recipient_email);

        // Build the email message dynamically.
        // The recipient is now the 'email' parameter.
        let email_message = Message::builder()
            .from(Mailbox::new(
                Option::from("Verified email no replay".to_owned()),
                config_service.email_address.parse().unwrap(),
            ))
            // Use environment-based recipient routing
            .to(recipient_email.parse().unwrap_or_else(|e| {
                panic!("Invalid recipient email '{}': {}", recipient_email, e)
            }))
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
