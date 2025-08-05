use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entity::enums::{UserRole, UserStatus};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users", schema_name = "auth")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub email: String,

    #[sea_orm(unique)]
    pub username: String,

    pub password_hash: String,

    pub first_name: Option<String>,

    pub last_name: Option<String>,

    pub role: UserRole,

    pub status: UserStatus,

    pub email_verified: bool,

    pub last_login: Option<DateTimeWithTimeZone>,

    pub created_at: DateTimeWithTimeZone,

    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tokens::Entity")]
    Tokens,

    #[sea_orm(has_many = "super::sessions::Entity")]
    Sessions,
}

impl Related<super::tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tokens.def()
    }
}

impl Related<super::sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Get the user's full name
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{first} {last}"),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }

    /// Check if the user is active and verified
    pub fn is_active_and_verified(&self) -> bool {
        self.status.is_active() && self.email_verified
    }

    /// Check if the user can login
    pub fn can_login(&self) -> bool {
        self.status.is_active() && !self.status.is_suspended()
    }

    /// Check if the user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Check if the user can moderate content
    pub fn can_moderate(&self) -> bool {
        self.role.can_moderate_content()
    }

    /// Check if the user can manage other users
    pub fn can_manage_users(&self) -> bool {
        self.role.can_manage_users()
    }

    /// Get display name (full name or username)
    pub fn display_name(&self) -> String {
        if self.first_name.is_some() || self.last_name.is_some() {
            self.full_name()
        } else {
            self.username.clone()
        }
    }

    /// Check if email needs verification
    pub fn needs_email_verification(&self) -> bool {
        !self.email_verified && matches!(self.status, UserStatus::PendingVerification)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct RegisterRequestBody {
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
