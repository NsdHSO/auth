use super::services::UsersService;
use crate::http_response::error_handler::CustomError;
use actix_web::{post, web, HttpResponse};
use sea_orm::DatabaseConnection;

#[post("/users")]
pub async fn users(
    service: web::Data<UsersService>,
    db_conn: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, CustomError> {
    let service_instance = UsersService::new(db_conn.get_ref());

    Ok(HttpResponse::Ok().body({}))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(users);
}
