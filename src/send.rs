use dotenv::dotenv;
use eyre::{eyre, ContextCompat, Result, WrapErr};
use lettre::smtp::authentication::Credentials;
use lettre::{SendableEmail, SmtpClient, SmtpTransport, Transport};
use std::env;

pub struct MailerClient {
    transport: SmtpTransport,
    user: String,
}

impl MailerClient {
    fn credentials() -> Result<(String, String, String)> {
        const ERR: &str = "Error finding env variable";
        let _ = dotenv().wrap_err("Error reading .env file")?;

        let server = env::var("SMTP_SERVER").wrap_err(ERR)?;
        let user = env::var("SMTP_USER").wrap_err(ERR)?;
        let password = env::var("SMTP_PASSWORD").wrap_err(ERR)?;
        Ok((server, user, password))
    }

    pub fn new() -> Result<Self> {
        let (server, user, password) =
            MailerClient::credentials().wrap_err("Error obtaining server credentials")?;

        let transport = SmtpClient::new_simple(&server)
            .unwrap()
            .credentials(Credentials::new(user.clone(), password))
            .transport();

        Ok(MailerClient { transport, user })
    }

    pub fn send_mail(&mut self, email: SendableEmail) -> Result<()> {
        let r = self.transport.send(email).wrap_err("Error sending mail")?;

        println!("Send email, message: {:?}", r.message);

        Ok(())
    }

    pub fn get_user(&self) -> String {
        let user = &self.user;
        user.clone()
    }
}
