use anyhow::{Result, Context};
use shuttle_runtime::SecretStore;
use std::{str::FromStr, fmt::Display};

pub struct Config {
    pub environment: String,
    pub secret_key: String,
    pub hash_cost: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub api_port: u16,
    pub db_user: String,
    pub db_pass: String,
    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
    pub domain: String,
    pub gmail_email: String,
    pub gmail_token_pass: String,
}

impl Config {
    pub fn from_secret_store(secrets: SecretStore) -> Result<Config> {
        let environment = get_secret_var(&secrets, "ENVIRONMENT")?;

        Ok(Config {
            environment,
            secret_key: get_secret_var(&secrets, "SECRET_KEY")?,
            hash_cost: get_secret_var(&secrets, "HASH_COST")?,
            jwt_issuer: get_secret_var(&secrets, "JWT_ISSUER")?,
            jwt_audience: get_secret_var(&secrets, "JWT_AUDIENCE")?,
            api_port: get_secret_var_as::<u16>(&secrets, "API_PORT")?,
            db_user: get_secret_var(&secrets, "DB_USER")?,
            db_pass: get_secret_var(&secrets, "DB_PASS")?,
            db_name: get_secret_var(&secrets, "DB_NAME")?,
            db_host: get_secret_var(&secrets, "DB_HOST")?,
            domain: get_secret_var(&secrets, "DOMAIN")?,
            db_port: get_secret_var_as::<u16>(&secrets, "DB_PORT")?,
            gmail_email: get_secret_var(&secrets, "GMAIL_EMAIL")?,
            gmail_token_pass: get_secret_var(&secrets, "GMAIL_TOKEN_PASS")?,
        })
    }

    pub fn build_db_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.db_user, self.db_pass, self.db_host, self.db_port, self.db_name
        )
    }
}

fn get_secret_var(secrets: &SecretStore, key: &str) -> Result<String> {
    secrets
        .get(key)
        .context(format!("{} must be set in the secret store", key))
}

fn get_secret_var_as<T>(secrets: &SecretStore, key: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let value = secrets
        .get(key)
        .context(format!("{} must be set in the secret store", key))?;

    value
        .parse::<T>()
        .map_err(|e| anyhow::anyhow!("Failed to parse secret {}: {}", key, e))
}
