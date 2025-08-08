use super::services::AuthService;
use crate::entity::users::AuthRequestBody;
use crate::http_response::error_handler::{CustomError, ValidatedJson};
use crate::http_response::http_response_builder;
use actix_web::{post, web, HttpResponse};
use actix_web::dev::ConnectionInfo;

#[post("/auth/register")]
pub async fn register(
    payload: ValidatedJson<AuthRequestBody>,
    service: web::Data<AuthService>,
    conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let registration = service.register(Option::from(payload.0),conn_info).await;
    match registration {
        Ok(user) => {
            let response = http_response_builder::ok(user);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Err(err),
    }
}

#[post("/auth/login")]
pub async fn login(
    payload: ValidatedJson<AuthRequestBody>,
    service: web::Data<AuthService>,
    conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let registration = service.login(payload.0,conn_info).await;
    match registration {
        Ok(user) => {
            let response = http_response_builder::ok(user);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Err(err),
    }
}
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(register);
    config.service(login);
}
