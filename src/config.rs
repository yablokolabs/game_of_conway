#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set".to_string())?,
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET must be set".to_string())?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|_| "PORT must be a valid u16".to_string())?,
            admin_username: std::env::var("ADMIN_USERNAME").ok(),
            admin_password: std::env::var("ADMIN_PASSWORD").ok(),
        })
    }
}
