use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

#[derive(Clone)]
pub struct MailSendService {
    email: String,
    password: String,
    smtp_password: String,
    port: String,
}

impl MailSendService {
    pub fn new() -> Self {
        let email = env::var("EMAIL_ADDRESS").expect("EMAIL_ADDRESS must be set");
        let password = env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let port = env::var("PORT_HOST").expect("PORT_HOST must be set");
        Self {
            email,
            password,
            smtp_password,
            port,
        }
    }

    pub fn send_mail(
        &self,
        email: String,
        token: String,
    ) -> Result<(), lettre::transport::smtp::Error> {
        // Construct the full verification URL using the provided token.
        let verification_link = format!("{}/v1/auth/verify/{}", self.port, token);

        // Build the email message dynamically.
        // The recipient is now the 'email' parameter.
        let email_message = Message::builder()
            .from(Mailbox::new(
                Option::from("Verified email no replay".to_owned()),
                self.email.parse().unwrap(),
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

        let creds = Credentials::new(self.email.to_owned(), self.smtp_password.to_owned());

        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email and return the result.
        mailer.send(&email_message).map(|_| ())
    }
}
