use super::services::AuthService;
use crate::components::users::UsersService;
use crate::entity::users::RegisterRequestBody;
use crate::http_response::error_handler::{CustomError, ValidatedJson};
use actix_web::{post, web, HttpResponse};
use sea_orm::DatabaseConnection;
#[post("/auth/register")]
pub async fn register(
    payload: ValidatedJson<RegisterRequestBody>,
    _service: web::Data<AuthService>,
    _service_user: web::Data<UsersService>,
    db_conn: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, CustomError> {
    let service_instance = AuthService::new(db_conn.get_ref(), _service_user.get_ref());
    let registration = service_instance
        .register(Option::from(payload.0))
        .await;
    match registration {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Err(err),
    }
}
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(register);
}
