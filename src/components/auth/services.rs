use std::future::Future;
use crate::components::users::UsersService;
use crate::entity::users::{Model, RegisterRequestBody};
use sea_orm::DatabaseConnection;
use crate::http_response::error_handler::CustomError;
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

    pub async fn register(&self, payload: Option<RegisterRequestBody>) -> Result<Option<Model>, CustomError> {
        let user = self.users_service.find(
            "email",
            crate::components::users::enums::SearchValue::String(payload.unwrap().email),
        ).await?; 
        
        Ok(user)
    }
}
