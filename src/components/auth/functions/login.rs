use crate::components::auth::functions::generate_jwt_token;
use crate::components::tokens::TokensService;
use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
use crate::config_service;
use crate::entity::tokens::ValueFilterBy;
use crate::entity::users::{ActiveModel, AuthRequestBody, AuthResponseBody, BodyToken};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use actix_web::dev::ConnectionInfo;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

pub async fn login_logic(
    users_service: &UsersService,
    payload: AuthRequestBody,
    conn_info: ConnectionInfo,
    conn: &DatabaseConnection,
    tokens_service: &TokensService,
) -> Result<Result<Option<AuthResponseBody>, CustomError>, CustomError> {
    let ip_address = conn_info
        .realip_remote_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let user = users_service
        .find("email", SearchValue::String(payload.email.to_string()))
        .await;
    let user_model = user?;
    let check_pass = users_service
        .check_credentials_and_email_verification(payload, ip_address, user_model)
        .await
        .unwrap_or_else(|value| match value {
            Ok(e) => Err(CustomError::new(
                HttpCodeW::Unauthorized,
                format!("Invalid credentials, {:?}", e),
            )),
            Err(e) => Err(CustomError::new(
                HttpCodeW::Unauthorized,
                format!("Invalid credentials, {:?}", e),
            )),
        });
    Ok(match check_pass {
        Ok(model) => {
            let active_model: ActiveModel = model;
            let update = active_model.update(conn).await;
            match update {
                Ok(update_model) => {
                    let jwt_token = generate_jwt_token(
                        update_model.id,
                        config_service().access_token_max_age,
                        config_service().access_token_private_key.to_owned(),
                    );
                    let(refresh_raw, _row) = tokens_service.create_refresh_token_for_user(update_model.id, config_service().refresh_token_max_age).await?;
                    match jwt_token {
                        Ok(token_details) => Ok(Some(AuthResponseBody {
                            body: BodyToken {
                                email: update_model.email.clone(),
                                access_token: token_details.token.unwrap_or_default(),
                                username: update_model.username.clone(),
                            },
                            refresh_token: refresh_raw,
                        })),
                        Err(e) => {
                            println!("JWT generation error: {:?}", e);
                            Err(CustomError::new(
                                HttpCodeW::InternalServerError,
                                "Failed to generate access token".to_string(),
                            ))
                        }
                    }
                }
                Err(_) => Err(CustomError::new(
                    HttpCodeW::InternalServerError,
                    "Failed to update user".to_string(),
                )),
            }
        }
        Err(e) => Err(e),
    })
}
