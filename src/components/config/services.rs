fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set", var_name))
}

#[derive(Debug, Clone)]
pub struct ConfigService {
    pub database_url: String,

    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,

    // Additional keys from your list
    pub rust_log: String,
    pub schema_synchronize: bool,
    pub host: String,
    pub port: u16,
    pub app_env: String,
    pub prod_database_url: String,
    pub synchronize: bool,
    pub auto_migrate: bool,
    pub email_address: String,
    pub email_password: String,
    pub smtp_password: String,
    pub smtp_transport: String,
    pub port_host: String,
}

impl ConfigService {
    pub fn new() -> Self {
        let database_url = get_env_var("DATABASE_URL");

        let access_token_private_key = get_env_var("ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_env_var("ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_env_var("ACCESS_TOKEN_EXPIRED_IN");
        let access_token_max_age = get_env_var("ACCESS_TOKEN_MAXAGE");
        let refresh_token_expires_in = get_env_var("REFRESH_TOKEN_EXPIRED_IN");
        let refresh_token_max_age = get_env_var("REFRESH_TOKEN_MAXAGE");
        // Additional keys from your list
        let rust_log = get_env_var("RUST_LOG");
        let schema_synchronize = get_env_var("SCHEMA_SYNCHRONIZE").parse::<bool>().unwrap();
        let host = get_env_var("HOST");
        let port = get_env_var("PORT").parse::<u16>().unwrap();
        let app_env = get_env_var("APP_ENV");
        let prod_database_url = get_env_var("PROD_DATABASE_URL");
        let synchronize = get_env_var("SYNCHRONIZE").parse::<bool>().unwrap();
        let auto_migrate = get_env_var("AUTO_MIGRATE").parse::<bool>().unwrap();
        let email_address = get_env_var("EMAIL_ADDRESS");
        let email_password = get_env_var("EMAIL_PASSWORD");
        let smtp_password = get_env_var("SMTP_PASSWORD");
        let smtp_transport = get_env_var("SMTP_TRANSPORT");
        let port_host = get_env_var("PORT_HOST");

        ConfigService {
            database_url,
            access_token_private_key,
            access_token_public_key,
            access_token_expires_in,
            access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
            refresh_token_max_age: refresh_token_max_age.parse::<i64>().unwrap(),
            refresh_token_expires_in,
            // Add the new keys here
            rust_log,
            schema_synchronize,
            host,
            port,
            app_env,
            prod_database_url,
            synchronize,
            auto_migrate,
            email_address,
            email_password,
            smtp_password,
            smtp_transport,
            port_host,
        }
    }
}
