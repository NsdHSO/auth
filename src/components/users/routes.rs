use super::services::UsersService;
use crate::http_response::error_handler::CustomError;
use actix_web::{post, web, HttpResponse};

#[post("/users")]
pub async fn users(_service: web::Data<UsersService>) -> Result<HttpResponse, CustomError> {
    let _service_instance = _service.get_ref();

    Ok(HttpResponse::Ok().body(()))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(users);
}
