use super::services::AuthService;
use crate::components::auth::functions::verify_jwt_token;
use crate::components::auth::local_enum::Info;
use crate::components::config::ConfigService;
use crate::config_service;
use crate::entity::tokens::{IntrospectRequest, IntrospectResponse};
use crate::entity::users::AuthRequestBody;
use crate::http_response::error_handler::{CustomError, ValidatedJson};
use crate::http_response::http_response_builder;
use actix_web::cookie::{time, Cookie, SameSite};
use actix_web::dev::ConnectionInfo;
use actix_web::{get, post, web, HttpRequest, HttpResponse};

#[post("/auth/register")]
pub async fn register(
    payload: ValidatedJson<AuthRequestBody>,
    service: web::Data<AuthService>,
    service_config: web::Data<ConfigService>,
    conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let registration = service
        .register(Option::from(payload.0), conn_info, &service_config)
        .await;
    match registration {
        Ok(user) => {
            let response = http_response_builder::ok(user);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Err(err),
    }
}


#[post("/auth/refresh")]
pub async fn refresh(
    req: HttpRequest,
    service: web::Data<AuthService>,
    _conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let refresh = service.refresh(req.cookie("refresh_token"));
    match refresh.await {
        Ok(user) => {
            let user = user.unwrap();
            let response = http_response_builder::ok(user.body);
            let refresh_cookie = Cookie::build("refresh_token", user.refresh_token)
                .path("/")
                .max_age(time::Duration::days(config_service().refresh_token_max_age))
                .same_site(SameSite::None)
                .http_only(true)
                .secure(true)
                .finish();
            Ok(HttpResponse::Ok().cookie(refresh_cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

#[post("/auth/login")]
pub async fn login(
    payload: ValidatedJson<AuthRequestBody>,
    service: web::Data<AuthService>,
    _conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let registration = service.login(payload.0, _conn_info).await;
    match registration {
        Ok(payload_auth) => {
            let payload_auth = payload_auth.unwrap();
            let response = http_response_builder::ok(payload_auth.body);
            let refresh_cookie = Cookie::build("refresh_token", payload_auth.refresh_token)
                .path("/")
                .max_age(time::Duration::days(config_service().refresh_token_max_age))
                .same_site(SameSite::None)
                .http_only(true)
                .secure(true)
                .finish();
            Ok(HttpResponse::Ok().cookie(refresh_cookie).json(response))
        }
        Err(err) => Err(err),
    }
}

#[get("/auth/verify/{token}")]
pub async fn verify_email(
    info: web::Path<Info>,
    service: web::Data<AuthService>,
    conn_info: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    let verified = service
        .verify_email(info.into_inner().token, conn_info)
        .await;
    match verified {
        Ok(user) => {
            let response = http_response_builder::ok(user);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Err(err),
    }
}

#[post("/auth/introspect")]
pub async fn introspect(
    payload: web::Json<IntrospectRequest>,
) -> Result<HttpResponse, CustomError> {
    let public_key_b64 = config_service().access_token_public_key.clone();
    match verify_jwt_token(public_key_b64, &payload.token) {
        Ok(details) => Ok(HttpResponse::Ok().json(IntrospectResponse {
            active: true,
            sub: Some(details.user_id.to_string()),
            token_uuid: Some(details.token_uuid.to_string()),
        })),
        Err(_) => Ok(HttpResponse::Ok().json(IntrospectResponse {
            active: false,
            sub: None,
            token_uuid: None,
        })),
    }
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(register);
    config.service(login);
    config.service(verify_email);
    config.service(refresh);
    config.service(introspect);
}
