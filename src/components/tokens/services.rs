use crate::components::users::UsersService;
use base64::engine::general_purpose;
use base64::Engine;
// For a specific base64 engine
use crate::entity::tokens::{ActiveModel, Model};
use crate::entity::TokenType::EmailVerification;
use crate::utils::helpers::now_date_time_utc;
// For base64 encoding
use chrono::Duration;
use general_purpose::URL_SAFE_NO_PAD;
use rand::random;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, Set};
use uuid::Uuid;

#[derive(Clone)]
pub struct TokensService {
    conn: DatabaseConnection,
    users_service: UsersService,
}

impl TokensService {
    pub fn new(conn: &DatabaseConnection, users_service: &UsersService) -> Self {
        Self {
            conn: conn.clone(),
            users_service: users_service.clone(),
        }
    }

    fn create_token(&self, user_id: Uuid) -> ActiveModel {
        let expires_at = now_date_time_utc() + Duration::hours(1);

        // Generate a 32-byte random token and encode it to Base64
        let random_bytes: [u8; 32] = random();
        let token_string = URL_SAFE_NO_PAD.encode(&random_bytes);
        ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            token: Set(token_string),
            token_type: Set(EmailVerification),
            expires_at: Set(DateTimeWithTimeZone::from(expires_at)),
            is_revoked: Set(false),
            created_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
            updated_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
        }
    }

    pub async fn create_token_for_user(&self, user_id: Uuid) -> Result<Model, DbErr> {
        let token = self.create_token(user_id);
        
        let active_model : ActiveModel= token.into();
        active_model.insert(&self.conn).await
    }
}
