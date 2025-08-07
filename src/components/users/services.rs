use crate::components::users::enums::SearchValue;
use crate::entity::users::{ActiveModel, Column, Entity, Model, RegisterRequestBody};
use crate::entity::UserRole::User;
use crate::entity::UserStatus;
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use crate::utils::helpers::{hash_password, now_date_time_utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;
use UserStatus::PendingVerification;

#[derive(Clone)]
pub struct UsersService {
    conn: DatabaseConnection,
}

impl UsersService {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }

    pub async fn find(&self, field: &str, value: SearchValue) -> Result<Model, CustomError> {
        let query = match (field, value) {
            ("id", SearchValue::Uuid(uuid)) => Entity::find_by_id(uuid),
            ("email", SearchValue::String(email)) => {
                // Destructure the enum to get the inner String and then borrow it.
                if !email_address::EmailAddress::is_valid(&email) {
                    // Return an error if the email is not valid.
                    return Err(CustomError::new(
                        HttpCodeW::BadRequest,
                        "Invalid email format".to_string(),
                    ));
                }
                // If the email is valid, build the query.
                Entity::find().filter(Column::Email.like(email))
            }

            e => {
                return Err(CustomError::new(
                    HttpCodeW::BadRequest,
                    "Invalid field or value".to_string(),
                ))
            }
        };

        let user = query
            .one(&self.conn)
            .await
            .map_err(|e| CustomError::new(HttpCodeW::InternalServerError, e.to_string()))?;

        match user {
            Some(user_model) => Ok(user_model),
            None => Err(CustomError::new(
                HttpCodeW::NotFound,
                "User not found".to_string(),
            )),
        }
    }
    pub async fn create(&self, payload: RegisterRequestBody) -> Result<Model, CustomError> {
        let active_model = Self::create_payload(payload);
        let result = active_model.insert(&self.conn).await;

        match result {
            Ok(res) => Ok(res),
            Err(e) => {
                Err(CustomError::new(
                    HttpCodeW::InternalServerError,
                    format!("Error creating user: {}", e),
                ))
            }
        }
    }

    pub fn create_payload(payload: RegisterRequestBody) -> ActiveModel {
        let hashed = hash_password(payload.password.as_str()).expect("hash failed");

        ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(payload.email),
            username: Set(payload.username),
            password_hash: Set(hashed),
            first_name: Set(payload.first_name),
            last_name: Set(payload.last_name),
            role: Set(Default::default()),
            status: Set(PendingVerification),
            email_verified: Set(false),
            last_login: Set(None),
            created_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
            updated_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
        }
    }
}
