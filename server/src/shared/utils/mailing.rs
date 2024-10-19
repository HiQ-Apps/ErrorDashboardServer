use lettre::message::{header, Mailbox, Message};
use lettre::{SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::error::Error as LettreError;
use crate::shared::utils::errors::{ServerError, ExternalError};
use crate::config::Config;

pub struct EmailContent {
    pub greeting: String,
    pub main_message: String,
    pub body: String,
    pub dynamic_content: Option<String>,
}

pub fn send_email(config: &Config, recipient: &str, subject: &str, content: &EmailContent) -> Result<(), ServerError> {
    // Parse the "from" and "to" addresses, mapping AddressError to your custom error type
    let from_address: Mailbox = config.gmail_email.parse()
        .map_err(|err| ServerError::ExternalError(ExternalError::Address(err)))?;
    
    let to_address: Mailbox = recipient.parse()
        .map_err(|err| ServerError::ExternalError(ExternalError::Address(err)))?;

    let styled_body = format!(
        r#"
        <html>
        <body style="font-family: Arial, sans-serif; background-color: #f4f4f4; padding: 20px;">
            <table style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 20px; border-radius: 8px; box-shadow: 0 0 10px rgba(0,0,0,0.1);">
                <tr>
                    <td style="text-align: center; padding-bottom: 20px;">
                        <h1 style="color: #333333;">{greeting}</h1>
                    </td>
                </tr>
                <tr>
                    <td style="padding: 20px; color: #555555; line-height: 1.6;">
                        <p>{main_message}</p>
                        <p>{body}</p>
                        {dynamic_content}
                    </td>
                </tr>
            </table>
        </body>
        </html>
        "#,
        greeting = content.greeting,
        main_message = content.main_message,
        body = content.body,
        dynamic_content = match &content.dynamic_content {
            Some(value) => format!(
                r#"<div style="background-color: #e0e0e0; padding: 10px; border-radius: 5px; text-align: center; font-weight: bold; letter-spacing: 1px; font-size: 16px;">{}</div>"#, 
                value
            ),
            None => "".to_string(),
        },
    );

    let email = Message::builder()
        .from(from_address)
        .to(to_address)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .body(styled_body)
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
