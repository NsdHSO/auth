use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
use crate::entity::users::{ActiveModel, AuthRequestBody, Model, RegisterResponseBody};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use sea_orm::{ActiveEnum, ActiveModelTrait, ActiveValue, DatabaseConnection, Unchanged};
use serde_json::json;
use serde_json::Value;
use ActiveValue::Set;
use actix_web::dev::ConnectionInfo;

#[derive(Clone)]
pub struct AuthService {
    conn: DatabaseConnection,
    users_service: UsersService,
}

impl AuthService {
    pub fn new(conn: &DatabaseConnection, users_service: &UsersService) -> Self {
        Self {
            conn: conn.clone(),
            users_service: users_service.clone(),
        }
    }

    pub async fn register(
        &self,
        payload: Option<AuthRequestBody>,
        conn_info: ConnectionInfo,
    ) -> Result<Option<RegisterResponseBody>, CustomError> {
        let payload = payload.ok_or_else(|| {
            CustomError::new(
                HttpCodeW::BadRequest,
                "Missing registration data".to_string(),
            )
        })?;

        // Check if user with this email already exists
        let existing_user = self
            .users_service
            .find("email", SearchValue::String(payload.email.clone()))
            .await;

        match existing_user {
            // User exists - return conflict error
            Ok(_) => Err(CustomError::new(
                HttpCodeW::Conflict,
                "User with this email already exists".to_string(),
            )),
            // User not found - good, we can create one
            Err(e) if e.error_status_code == HttpCodeW::NotFound => {
                self.users_service.create(payload, conn_info).await.map(|model| {
                    Some(RegisterResponseBody {
                        user_id: model.id.to_string(),
                        email: model.email,
                        status: model.status.to_value(),
                    })
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn login(
        &self,
        payload: AuthRequestBody,
        conn_info: ConnectionInfo,
    ) -> Result<Option<Model>, CustomError> {
        let ip_address = conn_info.realip_remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let user = self
            .users_service
            .find("email", SearchValue::String(payload.email.to_string()))
            .await;
        let user_model = user?;
        let check_pass = match self.users_service.check_credentials_and_email_verification(payload, ip_address, user_model).await {
            Ok(value) => value,
            Err(value) => return value,
        };
        match check_pass {
            Ok(model) => {
                let active_model: ActiveModel = model.into();
                let update = active_model.update(&self.conn).await;
                match update {
                    Ok(update_model) => Ok(Some(update_model)),
                    Err(_) => Err(CustomError::new(
                        HttpCodeW::InternalServerError,
                        "Failed to update user".to_string(),
                    )),
                }
            }
            Err(e) => Err(e),
        }
    }
}
