use super::services::UsersService;
use crate::entity::users::UserSearchBody;
use crate::http_response::error_handler::{CustomError, ValidatedJson};
use crate::http_response::prepared_response::check_response_ok_or_return_error;
use actix_web::{get, post, web, HttpResponse};

#[post("/users")]
pub async fn users(_service: web::Data<UsersService>) -> Result<HttpResponse, CustomError> {
    let _service_instance = _service.get_ref();

    Ok(HttpResponse::Ok().body(()))
}
#[get("/users")]
pub async fn get_users(
    payload: ValidatedJson<UserSearchBody>,
    _service: web::Data<UsersService>,
) -> Result<HttpResponse, CustomError> {
    let service_instance = _service.get_ref();

    let fetched_users = service_instance.get_all(&payload.0).await;
    check_response_ok_or_return_error(fetched_users)
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(users);
    config.service(get_users);
}
