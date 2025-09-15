use sea_orm::entity::prelude::*;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::entity::enums::{UserRole, UserStatus};
use crate::utils::helpers::now_date_time_utc;

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

    #[serde(skip_serializing_if = "Option::is_none")]
    #[sea_orm(ignore)]
    pub search_tsv: Option<String>,

    pub last_name: Option<String>,

    pub role: UserRole,

    pub status: UserStatus,

    pub email_verified: bool,

    pub last_login: Option<DateTimeWithTimeZone>,

    pub created_at: DateTimeWithTimeZone,

    pub updated_at: DateTimeWithTimeZone,

    #[sea_orm(column_type = "JsonBinary")]
    pub login_history: Json,
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

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    // The `pre_save` method is called before an insert or update.
    // It returns the `ActiveModel` with any changes applied.
    async fn before_save<C>(self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut active_model = self;

        // The logic for updating your fields is correct.
        active_model.last_login = Set(Some(DateTimeWithTimeZone::from(now_date_time_utc())));
        active_model.updated_at = Set(DateTimeWithTimeZone::from(now_date_time_utc()));

        Ok(active_model)
    }
}

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
pub struct AuthRequestBody {
    pub email: String,
    pub username: Option<String>,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponseBody {
    pub body: BodyToken,
    pub refresh_token: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct BodyToken {
    pub email: String,
    pub username: String,
    pub access_token: String,
}
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct RegisterResponseBody {
    pub user_id: String,
    pub email: String,
    pub status: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct UserSearchBody {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct UserSearchResponseBody {
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}
impl From<Model> for UserSearchResponseBody {
    fn from(user: Model) -> Self {
        UserSearchResponseBody {
            username: user.username,
            first_name: user.first_name.unwrap_or_default(),
            email: user.email,
            last_name: user.last_name.unwrap_or_default(),
        }
    }
}
