use actix_web::{
    dev::Payload,
    error::{JsonPayloadError},
    web,
    FromRequest,
    HttpRequest,
    HttpResponse,
    ResponseError,
    http::StatusCode,
};
use futures_util::future::{LocalBoxFuture, ready};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

// Assuming you have this module for your custom HTTP response.
// This is not provided, but is necessary for the code to compile.
// For example:
//
// mod http_response {
//     use serde::Serialize;
//     #[derive(Debug, Serialize)]
//     pub struct ResponseObject {
//         pub message: String,
//     }
//     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
//     pub enum HttpCodeW {
//         BadRequest = 400,
//         NotFound = 404,
//         InternalServerError = 500,
//     }
//     pub fn create_response(message: String, _code: HttpCodeW) -> ResponseObject {
//         ResponseObject { message }
//     }
// }
//
use crate::http_response::{create_response, HttpCodeW};

// This is your CustomError struct and its implementations.
// It is now integrated into the document.
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomError {
    pub error_status_code: HttpCodeW,
    pub error_message: String,
}

impl CustomError {
    pub fn new(error_status_code: HttpCodeW, error_message: String) -> CustomError {
        CustomError {
            error_status_code,
            error_message,
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

// Implement std::error::Error for CustomError
impl StdError for CustomError {}

// Implement From for SeaORM's DbErr
impl From<DbErr> for CustomError {
    fn from(error: DbErr) -> CustomError {
        match error {
            DbErr::Conn(e) => {
                let msg = format!("Auth database connection error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            DbErr::Exec(e) => {
                let msg = format!("Auth database execution error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            DbErr::Query(e) => {
                let msg = format!("Auth database query error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            DbErr::Json(e) => {
                let msg = format!("Auth JSON error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            DbErr::ConvertFromU64(e) => {
                let msg = format!("Auth conversion error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            DbErr::RecordNotFound(_) => {
                CustomError::new(HttpCodeW::NotFound, "Auth record not found".to_string())
            } // Not an error that needs logging at ERROR level
            DbErr::Custom(e) => {
                let msg = format!("Custom auth database error: {e}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
            _ => {
                let msg = format!("Unknown auth database error: {error:?}");
                print!("{msg}"); // Log the error
                CustomError::new(HttpCodeW::InternalServerError, msg)
            }
        }
    }
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        // Log the error when it's being converted to an HTTP response
        print!(
            "Auth service responding with error: Status={:?}, Message={}",
            self.error_status_code, self.error_message
        );

        // Create a ResponseObject using the error message and mapped HttpCodeW
        let response_object = create_response(self.error_message.clone(), self.error_status_code);
        println!("Auth ResponseObject: {response_object:?}");
        // Build the HttpResponse based on the HttpCodeW
        let status_code = StatusCode::from_u16(self.error_status_code as u16)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        HttpResponse::build(status_code).json(response_object)
    }
}

// Our custom extractor struct. It's generic over the inner type `T`.
pub struct ValidatedJson<T>(pub T);

// Implement the `FromRequest` trait for our new `ValidatedJson` struct.
// This is what makes it an Actix-Web extractor.
impl<T: serde::de::DeserializeOwned + 'static> FromRequest for ValidatedJson<T> {
    // The type of error this extractor can return.
    // We will return our `CustomError` type.
    type Error = CustomError;
    // The type of future this method returns. We use LocalBoxFuture to handle
    // the async nature of the standard `web::Json` extractor.
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    // This is the main function that handles the extraction logic.
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // Use the standard `web::Json` extractor internally.
        let json_fut = web::Json::<T>::from_request(req, payload);

        // We wrap the logic in a box to satisfy the `LocalBoxFuture` type.
        Box::pin(async move {
            // Await the result of the `web::Json` extractor.
            match json_fut.await {
                Ok(json) => {
                    // If successful, wrap the value in our custom struct.
                    Ok(ValidatedJson(json.into_inner()))
                }
                Err(err) => {
                    // If it's a deserialization error, convert it to our
                    // custom error type.
                    let message = match err {
                        _ => format!("JSON payload error: {}", err),
                    };
                    // Return a `CustomError` with a 400 Bad Request status.
                    Err(CustomError::new(
                        HttpCodeW::BadRequest,
                        message,
                    ))
                }
            }
        })
    }
}


