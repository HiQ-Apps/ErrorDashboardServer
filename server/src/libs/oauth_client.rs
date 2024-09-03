use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use reqwest::{Client, Error};

use crate::config::Config;
use shared_types::user_dtos::GoogleUserInfoDTO;

pub fn create_oauth_client(config: &Config) -> BasicClient {
    let client_id = ClientId::new(config.google_client_id.clone());
    let client_secret = ClientSecret::new(config.google_secret_key.clone());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap();
    let redirect_url = RedirectUrl::new("http://localhost:8000/api/auth/callback".to_string()).unwrap();

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}


pub async fn fetch_google_user_info(access_token: &str) -> Result<GoogleUserInfoDTO, Error> {
    let client = Client::new();
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?;
    
    let user_info = response.json::<GoogleUserInfoDTO>().await?;

    Ok(user_info)
}
