use crate::components::tokens::TokensService;
use crate::config_service;
use crate::entity::users::{AuthResponseBody, BodyToken};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use crate::http_response::HttpCodeW::InternalServerError;
use actix_web::cookie::Cookie;
use sea_orm::{DatabaseConnection, TransactionTrait};
use crate::components::auth::functions::generate_jwt_token;
use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;

pub async fn refresh_logic(
    tokens_service: &TokensService,
    users_service: &UsersService,
    conn: &DatabaseConnection,
    cookie_refresh_token: Option<Cookie<'_>>,
) -> Result<Option<AuthResponseBody>, CustomError> {
    let refresh_token = match cookie_refresh_token {
        None => return Err(CustomError::new(HttpCodeW::Unauthorized, "Missing refresh token".to_string())),
        Some(v) => v.value().to_string(),
    };

    let txn = match conn.begin().await {
        Ok(t) => t,
        Err(e) => return Err(CustomError::new(InternalServerError, format!("Txn begin error: {e}"))),
    };
    let old_token_model = match tokens_service.is_token_available(&refresh_token, &txn).await {
        Ok(Some(model)) => model,
        Ok(None) => {
            return Err(CustomError::new(
                HttpCodeW::Unauthorized,
                "Invalid refresh token".into(),
            ));
        }
        Err(err) => {
            return Err(err);
        }
    };

    let user_id = old_token_model.user_id;


    if let Err(e) = TokensService::revoke_token(old_token_model, &txn).await {
        let _ = txn.rollback().await;
        return Err(CustomError::new(InternalServerError, format!("Token revoke error: {e}")));
    }

    let (new_raw_refresh, _new_row) = match tokens_service
        .create_refresh_token_for_user_txn(user_id, config_service().refresh_token_max_age, &txn)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(CustomError::new(InternalServerError, format!("Create new refresh token error: {e}")));
        }
    };

    let jwt = match generate_jwt_token(
        user_id,
        config_service().access_token_max_age,
        config_service().access_token_private_key.to_owned(),
    ) {
        Ok(v) => v,
        Err(_) => {
            let _ = txn.rollback().await;
            return Err(CustomError::new(InternalServerError, "Failed to generate access token".to_string()));
        }
    };

    let user = match users_service.find("id", SearchValue::Uuid(user_id)).await {
        Ok(u) => u,
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(CustomError::new(InternalServerError, format!("Failed to fetch user: {e}")));
        }
    };

    if let Err(e) = txn.commit().await {
        return Err(CustomError::new(InternalServerError, format!("Txn commit error: {e}")));
    }

    Ok(Some(AuthResponseBody {
        body: BodyToken {
            email: user.email,
            username: user.username,
            access_token: jwt.token.unwrap_or_default(),
        },
        refresh_token: new_raw_refresh,
    }))
}