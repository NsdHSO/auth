use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "ADMIN")]
    Admin,
    #[sea_orm(string_value = "USER")]
    User,
    #[sea_orm(string_value = "MODERATOR")]
    Moderator,
    #[sea_orm(string_value = "GUEST")]
    Guest,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "ADMIN",
            UserRole::User => "USER",
            UserRole::Moderator => "MODERATOR",
            UserRole::Guest => "GUEST",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn is_moderator_or_admin(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Moderator)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_moderate_content(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Moderator)
    }
}
