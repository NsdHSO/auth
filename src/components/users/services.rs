use crate::components::users::enums::SearchValue;
use crate::entity::users::{ActiveModel, AuthRequestBody, Column, Entity, Model};
use crate::entity::UserStatus;
use crate::entity::UserStatus::Active;
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use crate::utils::helpers::{hash_password, now_date_time_utc, verify_password};
use actix_web::dev::ConnectionInfo;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Unchanged,
};
use serde_json::{json, Value};
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
    pub async fn create(
        &self,
        payload: AuthRequestBody,
        conn_info: ConnectionInfo,
    ) -> Result<Model, CustomError> {
        let active_model = Self::create_payload(payload, conn_info);
        let result = active_model.insert(&self.conn).await;

        match result {
            Ok(res) => Ok(res),
            Err(e) => Err(CustomError::new(
                HttpCodeW::InternalServerError,
                format!("Error creating user: {}", e),
            )),
        }
    }

    pub async fn check_credentials(
        &self,
        payload: AuthRequestBody,
        user_model: &Model,
    ) -> Result<ActiveModel, CustomError> {
        match user_model.can_login() {
            true => match verify_password(payload.password.as_str(), &user_model.password_hash) {
                Ok(value) => match value {
                    true => {
                        let active_user: ActiveModel = user_model.clone().into();

                        Ok(active_user)
                    }
                    false => Err(CustomError::new(
                        HttpCodeW::Unauthorized,
                        "Invalid credentials".to_string(),
                    )),
                },
                Err(_) => Err(CustomError::new(
                    HttpCodeW::Unauthorized,
                    "Invalid credentials".to_string(),
                )),
            },
            false => Err(CustomError::new(
                HttpCodeW::Unauthorized,
                "Invalid credentials".to_string(),
            )),
        }
    }

    pub async fn check_credentials_and_email_verification(
        &self,
        payload: AuthRequestBody,
        ip_address: &String,
        user_model: Model,
    ) -> Result<Result<ActiveModel, CustomError>, Result<Option<Model>, CustomError>> {
        if (user_model.needs_email_verification()) {
            let mut active_user: ActiveModel = user_model.into();
            let new_login = json!({
                "timestamp": now_date_time_utc(),
                "notes": "Needs email verification",
                "ip_address": ip_address,
            });
            Self::add_details_login(&mut active_user, new_login);
            active_user
                .update(&self.conn)
                .await
                .expect("Failed to update user");

            return Err(Err(CustomError::new(
                HttpCodeW::Unauthorized,
                "User needs email verification".to_string(),
            )));
        }
        let check_pass = self.check_credentials(payload, &user_model).await;
        Ok(check_pass)
    }

    pub fn add_details_login(active_user: &mut ActiveModel, new_login: Value) {
        let mut login_history: Vec<Value> = match &active_user.login_history {
            Unchanged(Value::Array(array)) => array.clone(),
            Set(Value::Array(array)) => array.clone(),
            _ => vec![],
        };
        login_history.push(new_login);
        active_user.login_history = Set(Value::Array(login_history));
    }

    pub async fn update(
        self,
        field: &str,
        value: &str,
        mut model: ActiveModel,
        ip_address: String,
    ) -> Result<(), CustomError> {
        match (field, value) {
            ("email_verified", va) => {
                let new_login = json!({
                    "timestamp": now_date_time_utc(),
                    "notes": "Email verified successfully",
                    "ip_address": ip_address,
                });
                Self::add_details_login(&mut model, new_login);
                model.email_verified = Set(true);
                model.status = Set(Active)
            }
            (&_, _) => todo!(),
        };

        model
            .update(&self.conn)
            .await
            .map_err(|e| CustomError::new(HttpCodeW::InternalServerError, e.to_string()))
            .map(|s| ())
    }

    pub fn create_payload(payload: AuthRequestBody, conn_info: ConnectionInfo) -> ActiveModel {
        let hashed = hash_password(payload.password.as_str()).expect("hash failed");
        let ip_address = conn_info
            .realip_remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let new_login = json!({
            "timestamp": now_date_time_utc(),
            "notes": "User was created",
            "ip_address": ip_address,
        });

        ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(payload.email),
            username: Set(payload.username.unwrap()),
            password_hash: Set(hashed),
            first_name: Set(payload.first_name),
            last_name: Set(payload.last_name),
            role: Set(Default::default()),
            status: Set(PendingVerification),
            email_verified: Set(false),
            last_login: Set(None),
            created_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
            updated_at: Set(DateTimeWithTimeZone::from(now_date_time_utc())),
            login_history: Set(Value::Array(vec![new_login])),
        }
    }
}
