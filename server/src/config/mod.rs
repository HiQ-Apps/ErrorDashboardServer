use anyhow::{Result, Context};
use std::{env, str::FromStr, fmt::Display};


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
}

impl Config {
    pub fn from_env() -> Result<Config> {
        dotenv::from_filename(".env").ok();

        let environment = get_env_var("ENVIRONMENT")?;

        let env_file = format!(".env.{}.local", environment);
        dotenv::from_filename(env_file).ok();

        Ok(Config {
            environment,
            secret_key: get_env_var("SECRET_KEY")?,
            hash_cost: get_env_var("HASH_COST")?,
            jwt_issuer: get_env_var("JWT_ISSUER")?,
            jwt_audience: get_env_var("JWT_AUDIENCE")?,
            api_port: get_env_var_as::<u16>("API_PORT")?,
            db_user: get_env_var("DB_USER")?,
            db_pass: get_env_var("DB_PASS")?,
            db_name: get_env_var("DB_NAME")?,
            db_host: get_env_var("DB_HOST")?,
            db_port: get_env_var_as::<u16>("DB_PORT")?,
        })
    }

    pub fn build_db_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.db_user, self.db_pass, self.db_host, self.db_port, self.db_name
        )
    }
}

fn get_env_var(key: &str) -> Result<String> {
    env::var(key).context(format!("{} must be set in the environment or .env file", key))
}

fn get_env_var_as<T>(key: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let value = env::var(key).context(format!("{} must be set in the environment or .env file", key))?;
    
    value.parse::<T>().map_err(|e| anyhow::anyhow!("Failed to parse environment variable {}: {}", key, e))
}
