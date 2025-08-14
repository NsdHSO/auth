use std::future::Future;
use actix_web::cookie::Cookie;
use crate::components::auth::functions::{login_logic, verify_jwt_token, TokenDetails};
use crate::components::config::ConfigService;
use crate::components::mail_send::MailSendService;
use crate::components::tokens::TokensService;
use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
use crate::entity::users::{AuthRequestBody, AuthResponseBody, RegisterResponseBody};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use actix_web::dev::ConnectionInfo;
use actix_web::web::service;
use jsonwebtoken::errors::Error;
use sea_orm::{ActiveEnum, DatabaseConnection, DatabaseTransaction, Iden, TransactionTrait};
use crate::config_service;
use crate::entity::tokens::Model;
use crate::http_response::HttpCodeW::InternalServerError;

#[derive(Clone)]
pub struct AuthService {
    conn: DatabaseConnection,
    users_service: UsersService,
    mail_send_service: MailSendService,
    tokens_service: TokensService,
}

impl AuthService {
    pub fn new(
        conn: &DatabaseConnection,
        users_service: &UsersService,
        tokens_service: &TokensService,
    ) -> Self {
        Self {
            conn: conn.clone(),
            users_service: users_service.clone(),
            mail_send_service: MailSendService::new(),
            tokens_service: tokens_service.clone(),
        }
    }

    pub async fn refresh(&self, cookie_refresh_token: Option<Cookie<'_>>) -> Result<Option<AuthResponseBody>, CustomError> {
        let refresh_token  =  match cookie_refresh_token {
            None => {
                return Err(CustomError::new(HttpCodeW::Unauthorized, "Missing refresh token".to_string()))
            }
            Some(v) =>
                v.value().to_string()

        };

        let old_token_model = match self.tokens_service.is_token_available(&refresh_token).await {
            Ok(Some(model)) => model,
            _ => {
                return Err(CustomError::new(HttpCodeW::Unauthorized, "Invalid refresh token".to_string()));
            }
        };

        let user_id = old_token_model.user_id;
        let txn = self.conn.begin().await.map_err(|e| {
            CustomError::new(InternalServerError, format!("Txn begin error: {e}"))
        })?;

        match TokensService::revoke_token(old_token_model, &txn).await {
            Ok(_) => {
                // If revoke_token succeeds, commit the transaction.
                txn.commit().await.map_err(|e| {
                    CustomError::new(InternalServerError, format!("Txn commit error: {e}"))
                })?;
            },
            Err(e) => {
                txn.rollback().await.expect("Failed to rollback transaction");
                return Err(CustomError::new(InternalServerError, format!("Token revoke error: {e}")));
            }
        };
        Ok(Some(AuthResponseBody{ body: Default::default(), refresh_token: format!("{:?}", refresh_token) }))

    }

    pub async fn register(
        &self,
        payload: Option<AuthRequestBody>,
        conn_info: ConnectionInfo,
        service_config: &ConfigService,
    ) -> Result<Option<RegisterResponseBody>, CustomError> {
        let payload = payload.ok_or_else(|| {
            CustomError::new(
                HttpCodeW::BadRequest,
                "Missing registration data".to_string(),
            )
        })?;

        // Check if user with this email already exists
        let existing_user = self
            .users_service
            .find("email", SearchValue::String(payload.email.clone()))
            .await;

        match existing_user {
            // User exists - return conflict error
            Ok(_) => Err(CustomError::new(
                HttpCodeW::Conflict,
                "User with this email already exists".to_string(),
            )),
            // User not found - good, we can create one
            Err(e) if e.error_status_code == HttpCodeW::NotFound => {
                let user_creation_result = self.users_service.create(payload, conn_info).await;

                // Then, process the result of user creation
                match user_creation_result {
                    Ok(model) => {
                        // If the user was created successfully, create the token
                        let token_creation_result =
                            self.tokens_service.create_token_for_user(model.id).await;

                        // Now, handle the result of token creation
                        match token_creation_result {
                            Ok(_token) => {
                                let _ = self.mail_send_service.send_mail(
                                    model.email.clone(),
                                    _token.token.clone(),
                                    service_config,
                                );
                                Ok(Some(RegisterResponseBody {
                                    user_id: model.id.to_string(),
                                    email: model.email,
                                    status: model.status.to_value(),
                                }))
                            }
                            Err(token_err) => Err(CustomError::new(
                                HttpCodeW::InternalServerError,
                                token_err.to_string(),
                            )),
                        }
                    }
                    Err(user_err) => Err(user_err),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn login(
        &self,
        payload: AuthRequestBody,
        conn_info: ConnectionInfo,
    ) -> Result<Option<AuthResponseBody>, CustomError> {
        login_logic(
            &self.users_service,
            payload,
            conn_info,
            &self.conn,
            &self.tokens_service,
        )
        .await?
    }

    pub async fn verify_email(
        &self,
        token: String,
        conn_info: ConnectionInfo,
    ) -> Result<String, CustomError> {
        let ip_address = conn_info
            .realip_remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let result = self
            .tokens_service
            .set_verified_email(token, ip_address)
            .await;

        match result {
            Ok(value) => Ok(value),
            Err(e) => match e.error_status_code {
                HttpCodeW::Forbidden => {
                    return Err(CustomError::new(
                        HttpCodeW::Forbidden,
                        "Token not found or None".to_string(),
                    ))
                }
                _ => Err(e),
            },
        }
    }
}
