use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
use crate::entity::users::{Model, RegisterRequestBody};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use sea_orm::DatabaseConnection;

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
        payload: Option<RegisterRequestBody>,
    ) -> Result<Option<Model>, CustomError> {
        let payload = payload.ok_or_else(|| {
            CustomError::new(HttpCodeW::BadRequest, "Missing registration data".to_string())
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
            Err(e) if e.error_status_code == HttpCodeW::NotFound => self
                .users_service
                .create(payload)
                .await
                .map(|model| Some(model)),
            // Other error - propagate
            Err(e) => Err(e),
        }
    }
}
