use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "token_type")]
pub enum TokenType {
    #[sea_orm(string_value = "ACCESS")]
    Access,

    #[sea_orm(string_value = "REFRESH")]
    Refresh,

    #[sea_orm(string_value = "RESET_PASSWORD")]
    ResetPassword,

    #[sea_orm(string_value = "EMAIL_VERIFICATION")]
    EmailVerification,
}

#[allow(dead_code)]
impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Access => "ACCESS",
            TokenType::Refresh => "REFRESH",
            TokenType::ResetPassword => "RESET_PASSWORD",
            TokenType::EmailVerification => "EMAIL_VERIFICATION",
        }
    }

    pub fn is_access_token(&self) -> bool {
        matches!(self, TokenType::Access)
    }

    pub fn is_refresh_token(&self) -> bool {
        matches!(self, TokenType::Refresh)
    }

    pub fn is_verification_token(&self) -> bool {
        matches!(self, TokenType::EmailVerification | TokenType::ResetPassword)
    }

    /// Get the default expiration time in minutes for each token type
    pub fn default_expiration_minutes(&self) -> i64 {
        match self {
            TokenType::Access => 15,           // 15 minutes
            TokenType::Refresh => 10080,       // 7 days
            TokenType::ResetPassword => 60,    // 1 hour
            TokenType::EmailVerification => 1440, // 24 hours
        }
    }
}
