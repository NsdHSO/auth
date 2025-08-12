// For base64 encoding
use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
// For a specific base64 engine
use crate::entity::tokens::{ActiveModel, Column, Entity, Model, ValueFilterBy};
use crate::entity::TokenType::{EmailVerification, Refresh};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use crate::utils::helpers::now_date_time_utc;
use crate::{config_service, entity};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Duration;
use general_purpose::URL_SAFE_NO_PAD;
use rand::random;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, Set,
};
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
            refresh_token: Set(
                crate::components::auth::functions::token::generate_jwt_token(
                    user_id,
                    config_service().refresh_token_max_age,
                    config_service().access_token_private_key.to_owned(),
                )
                .unwrap()
                .token
                .expect("REASON"),
            ),
        }
    }

    pub async fn create_token_for_user(&self, user_id: Uuid) -> Result<Model, DbErr> {
        let token = self.create_token(user_id);

        let active_model: ActiveModel = token.into();
        active_model.insert(&self.conn).await
    }

    pub async fn find_by(&self, field: &str, value: ValueFilterBy) -> Result<Model, CustomError> {
        let query = match (field, value) {
            ("user_id", ValueFilterBy::Uuid(value_uid)) => {
                Entity::find().filter(Column::UserId.eq(value_uid))
            }
            _ => {
                return Err(CustomError::new(
                    HttpCodeW::BadRequest,
                    "Invalid field or value".to_string(),
                ));
            }
        };

        query
            .one(&self.conn)
            .await
            .map(|model| model.unwrap())
            .map_err(|e| {
                CustomError::new(
                    HttpCodeW::InternalServerError,
                    format!("Database error: {}", e),
                )
            })
    }

    pub async fn set_verified_email(
        &self,
        token: String,
        ip_address: String,
    ) -> Result<String, CustomError> {
        let query = Entity::find()
            .filter(Column::Token.like(token))
            .filter(Column::IsRevoked.eq(false));
        let token_wrapper = query.one(&self.conn).await;

        match token_wrapper {
            Ok(Some(response_model)) => {
                let user_result = self
                    .users_service
                    .find("id", SearchValue::Uuid(response_model.user_id))
                    .await;

                match user_result {
                    Ok(user_model) => {
                        let active_user_model: entity::users::ActiveModel =
                            user_model.into_active_model();

                        // Await the user update directly
                        let updated_user_result = self
                            .users_service
                            .clone()
                            .update("email_verified", "true", active_user_model, ip_address)
                            .await;
                        if let Ok(_) = updated_user_result {
                            // If user update was successful, update the token
                            let mut active_token_model: ActiveModel = response_model.into();
                            active_token_model.is_revoked = Set(true);
                            active_token_model.token_type = Set(Refresh);

                            match active_token_model.update(&self.conn).await {
                                Ok(_) => Ok("Email verified successfully".to_string()),
                                Err(e) => Err(CustomError::new(
                                    HttpCodeW::InternalServerError,
                                    format!("Failed to revoke token: {}", e),
                                )),
                            }
                        } else {
                            Err(CustomError::new(
                                HttpCodeW::InternalServerError,
                                "Failed to update user's email verification status".to_string(),
                            ))
                        }
                    }
                    Err(e) => Err(CustomError::new(
                        HttpCodeW::NotFound,
                        format!("User not found: {}", e),
                    )),
                }
            }
            Ok(e) => Err(CustomError::new(
                HttpCodeW::Forbidden,
                format!("Token not found or {:?}", e).to_string(),
            )),
            Err(e) => Err(CustomError::new(
                HttpCodeW::NotFound,
                format!("Database error: {}", e),
            )),
        }
    }
}
