use super::services::AuthService;
use crate::entity::users::RegisterRequestBody;
use crate::http_response::error_handler::{CustomError, ValidatedJson};
use crate::http_response::http_response_builder;
use actix_web::{post, web, HttpResponse};

#[post("/auth/register")]
pub async fn register(
    payload: ValidatedJson<RegisterRequestBody>,
    service: web::Data<AuthService>,
) -> Result<HttpResponse, CustomError> {
    let registration = service.register(Option::from(payload.0)).await;
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
}
