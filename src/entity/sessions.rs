use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[sea_orm(unique)]
    pub session_token: String,

    pub ip_address: Option<String>,

    pub user_agent: Option<String>,

    pub expires_at: DateTimeWithTimeZone,

    pub is_active: bool,

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

impl Model {
    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().with_timezone(&chrono::Utc) > self.expires_at.with_timezone(&chrono::Utc)
    }

    /// Check if the session is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Get remaining validity time in seconds
    pub fn remaining_validity_seconds(&self) -> i64 {
        if self.is_expired() || !self.is_active {
            0
        } else {
            (self.expires_at.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_seconds()
        }
    }

    /// Check if session will expire within given minutes
    pub fn expires_within_minutes(&self, minutes: i64) -> bool {
        let threshold = chrono::Utc::now() + chrono::Duration::minutes(minutes);
        self.expires_at.with_timezone(&chrono::Utc) <= threshold
    }

    /// Get a safe representation of user agent (truncated if too long)
    pub fn safe_user_agent(&self) -> Option<String> {
        self.user_agent.as_ref().map(|ua| {
            if ua.len() > 200 {
                format!("{}...", &ua[..197])
            } else {
                ua.clone()
            }
        })
    }

    /// Check if the session was created from the same IP
    pub fn is_same_ip(&self, ip: &str) -> bool {
        self.ip_address.as_ref().map_or(false, |session_ip| session_ip == ip)
    }

    /// Get session duration in minutes since creation
    pub fn duration_minutes(&self) -> i64 {
        (chrono::Utc::now() - self.created_at.with_timezone(&chrono::Utc)).num_minutes()
    }
}
