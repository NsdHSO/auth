use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entity::enums::TokenType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[sea_orm(unique)]
    pub token: String,

    pub token_type: TokenType,

    pub expires_at: DateTimeWithTimeZone,

    pub is_revoked: bool,

    pub created_at: DateTimeWithTimeZone,

    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[allow(dead_code)]
impl Model {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().with_timezone(&chrono::Utc) > self.expires_at.with_timezone(&chrono::Utc)
    }

    /// Check if the token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked
    }

    /// Check if the token is an access token
    pub fn is_access_token(&self) -> bool {
        self.token_type.is_access_token()
    }

    /// Check if the token is a refresh token
    pub fn is_refresh_token(&self) -> bool {
        self.token_type.is_refresh_token()
    }

    /// Check if the token is a verification token (email or password reset)
    pub fn is_verification_token(&self) -> bool {
        self.token_type.is_verification_token()
    }

    /// Get remaining validity time in seconds
    pub fn remaining_validity_seconds(&self) -> i64 {
        if self.is_expired() || self.is_revoked {
            0
        } else {
            (self.expires_at.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_seconds()
        }
    }

    /// Check if token will expire within given minutes
    pub fn expires_within_minutes(&self, minutes: i64) -> bool {
        let threshold = chrono::Utc::now() + chrono::Duration::minutes(minutes);
        self.expires_at.with_timezone(&chrono::Utc) <= threshold
    }
}
