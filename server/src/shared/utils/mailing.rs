use lettre::message::{header, Mailbox, Message};
use lettre::{SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::error::Error as LettreError;
use crate::shared::utils::errors::{ServerError, ExternalError};
use crate::config::Config;

pub fn send_email(config: &Config, recipient: &str, subject: &str, body: &str) -> Result<(), ServerError> {
    // Parse the "from" and "to" addresses, mapping AddressError to your custom error type
    let from_address: Mailbox = config.gmail_email.parse()
        .map_err(|err| ServerError::ExternalError(ExternalError::Address(err)))?;
    
    let to_address: Mailbox = recipient.parse()
        .map_err(|err| ServerError::ExternalError(ExternalError::Address(err)))?;

    let email = Message::builder()
        .from(from_address)
        .to(to_address)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .body(body.to_string())
        .map_err(|err| ServerError::ExternalError(ExternalError::Lettre(LettreError::from(err))))?;

    let creds = Credentials::new(config.gmail_email.clone(), config.gmail_token_pass.clone());

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .map_err(|err| ServerError::ExternalError(ExternalError::Smtp(err)))?
        .credentials(creds)
        .build();

    // Send the email and handle errors
    mailer.send(&email).map_err(|err| ServerError::ExternalError(ExternalError::Smtp(err)))?;

    Ok(())
}
