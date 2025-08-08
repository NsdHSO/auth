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
}

impl MailSendService {
    pub fn new() -> Self {
        let email = env::var("EMAIL_ADDRESS").expect("EMAIL_ADDRESS must be set");
        let password = env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        Self { email, password , smtp_password}
    }

    pub fn send_mail(&self, email:String) {
        let email = Message::builder()
            .from(Mailbox::new(
                Option::from("Auth Hospital".to_owned()), 
                self.email.parse().unwrap(),
            ))
            .to(
                "nechiforelsamuel@gmail.com".parse().unwrap(), // IMPORTANT: Updated recipient's email
            )
            .subject("Verify your email")
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Be happy!"))
            .unwrap();

        let creds = Credentials::new(self.email.to_owned(), self.smtp_password.to_owned());

        // Open a remote connection to Gmail's SMTP server
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {e:?}"),
        }
    }
}
