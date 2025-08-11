use crate::components::mail_send::MailSendService;
use crate::components::tokens::TokensService;
use crate::components::users::enums::SearchValue;
use crate::components::users::UsersService;
use crate::entity::users::{ActiveModel, AuthRequestBody, Model, RegisterResponseBody};
use crate::http_response::error_handler::CustomError;
use crate::http_response::HttpCodeW;
use actix_web::dev::ConnectionInfo;
use sea_orm::{ActiveEnum, ActiveModelTrait, DatabaseConnection};

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

    pub async fn register(
        &self,
        payload: Option<AuthRequestBody>,
        conn_info: ConnectionInfo,
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
                                let _ = self.mail_send_service
                                    .send_mail(model.email.clone(), _token.token.clone());
                                Ok(Some(RegisterResponseBody {
                                    user_id: model.id.to_string(),
                                    email: model.email,
                                    status: model.status.to_value(),
                                }))
                            }
                            Err(token_err) => {
                                // If token creation failed, return the error
                                Err(CustomError::new(
                                    HttpCodeW::InternalServerError,
                                    token_err.to_string(),
                                ))
                            }
                        }
                    }
                    Err(user_err) => {
                        // If user creation failed, return the error
                        Err(user_err)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn login(
        &self,
        payload: AuthRequestBody,
        conn_info: ConnectionInfo,
    ) -> Result<Option<Model>, CustomError> {
        let ip_address = conn_info
            .realip_remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let user = self
            .users_service
            .find("email", SearchValue::String(payload.email.to_string()))
            .await;
        let user_model = user?;
        let check_pass = match self
            .users_service
            .check_credentials_and_email_verification(payload, ip_address, user_model)
            .await
        {
            Ok(value) => value,
            Err(value) => return value,
        };
        match check_pass {
            Ok(model) => {
                let active_model: ActiveModel = model;
                let update = active_model.update(&self.conn).await;
                match update {
                    Ok(update_model) => Ok(Some(update_model)),
                    Err(_) => Err(CustomError::new(
                        HttpCodeW::InternalServerError,
                        "Failed to update user".to_string(),
                    )),
                }
            }
            Err(e) => Err(e),
        }
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
        let result = self.tokens_service.find_token(token, ip_address).await;

        match result {
            Ok(value) => Ok(value),
            Err(e) => match e.error_status_code {
                HttpCodeW::Forbidden => {
                    return Err(CustomError::new(HttpCodeW::Forbidden, "Token not found or None".to_string()))
                }
                _ => Err(e), // Or handle other error types here
            },
        }
    }
}