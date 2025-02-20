use serenity::async_trait;
use serenity::client::Client;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::sync::Arc;

use crate::shared::utils::errors::{ExternalError, ServerError};

#[derive(Clone)]
pub struct DiscordHandler {
    pub http: Arc<Http>,
}

// EventHandler impl necessary for Serenity
#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Discord bot connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Example: respond to a "!ping" command
        if msg.content == "!ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }
}

impl DiscordHandler {
    pub async fn new(token: &str) -> Result<Self, ServerError> {
        let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;
        let mut client = Client::builder(token, intents)
            .event_handler(DiscordHandler {
                http: Arc::new(Http::new(token)),
            })
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::Serenity(err)))?;

        // Clone the HTTP handle for later use
        let http = client.http.clone();

        // Spawn the client to run the event loop
        tokio::spawn(async move {
            if let Err(e) = client.start().await {
                eprintln!("Discord client error: {:?}", e);
            }
        });

        Ok(DiscordHandler { http })
    }

    pub async fn send_discord_alert(
        &self,
        discord_channel: u64,
        alert: &str,
    ) -> Result<(), ServerError> {
        ChannelId::new(discord_channel)
            .say(&self.http, alert)
            .await
            .map_err(|err| ServerError::ExternalError(ExternalError::Serenity(err)))?;
        Ok(())
    }
}
