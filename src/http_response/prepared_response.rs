use crate::http_response::error_handler::CustomError;
use crate::http_response::http_response_builder;
use actix_web::HttpResponse;
use serde::Serialize;

/// A generic helper function to convert a `Result` into a standardized `HttpResponse` or pass through an error.
///
/// This function is designed for use in `actix-web` handlers to reduce boilerplate code for success and
/// error responses. It takes a `Result` from a service or business logic layer. If the result is `Ok`,
/// it constructs an `HttpResponse` with a 200 OK status and a standardized JSON body. If the result is `Err`,
/// it simply returns the `CustomError`, allowing Actix to handle it.
///
/// # Arguments
///
/// * `response_result`: A `Result<T, CustomError>` where `T` is the data payload to be serialized.
///   - The `Ok` variant contains the data (`T`) that will be wrapped in the JSON response.
///   - The `Err` variant contains a `CustomError` that is propagated.
///
/// # Type Parameters
///
/// * `T`: The type of the successful response payload. This type must implement the
///   `serde::Serialize` trait to be converted into JSON.
///
/// # Returns
///
/// A `Result<HttpResponse, CustomError>`
/// - `Ok(HttpResponse)`: An HTTP response with a 200 OK status code.
/// - `Err(CustomError)`: The original error from the input `Result`.
///
/// # Examples
///
/// ```rust
/// use actix_web::{HttpResponse, web};
/// use serde::Serialize;
/// use crate::http_response::error_handler::CustomError;
/// use crate::http_response::http_response_builder;
///
/// #[derive(Serialize)]
/// pub struct UserProfile {
///     pub id: i32,
///     pub username: String,
/// }
///
/// /// A mock service function that returns either a successful result or an error.
/// async fn get_user_from_db(user_id: i32) -> Result<UserProfile, CustomError> {
///     if user_id > 0 {
///         Ok(UserProfile { id: user_id, username: "test_user".to_string() })
///     } else {
///         Err(CustomError::NotFound("User not found".to_string()))
///     }
/// }
///
/// /// An actix-web handler that uses the helper function.
/// pub async fn get_user_handler(
///     path: web::Path<i32>,
/// ) -> Result<HttpResponse, CustomError> {
///     let user_id = path.into_inner();
///
///     // The service call returns a Result
///     let user_result = get_user_from_db(user_id).await;
///
///     // The helper function handles the conversion to HttpResponse for us
///     check_response_ok_or_return_error(user_result)
/// }
/// ```
pub fn check_response_ok_or_return_error<T: Serialize>(
    response_result: Result<T, CustomError>,
) -> Result<HttpResponse, CustomError> {
    match response_result {
        Ok(response) => {
            let response = http_response_builder::ok(response);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => Err(err),
    }
}
