use crate::components::users::enums::SearchValue;
use crate::entity::users::{Column, Entity, Model};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Clone)]
pub struct UsersService {
    conn: DatabaseConnection,
}

impl UsersService {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }

    pub async fn find(
        &self,
        field: &str,
        value: SearchValue,
    ) -> Result<Option<Model>, CustomError> {
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
        if let Some(user_model) = user {
            Ok(Some(user_model))
        } else {
            Err(CustomError::new(
                HttpCodeW::NotFound,
                "User not found".to_string(),
            ))
        }
    }
}
