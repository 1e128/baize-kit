use std::error::Error;

use axum::response::{IntoResponse, Response};
use axum::Json;
use derive_more::From;

use crate::response::Reply;

pub trait ErrorCode {
    fn code(&self) -> i32 {
        500
    }
}

#[derive(From)]
pub struct ApiError<T>(pub T)
where
    T: ErrorCode + Error;

impl<T> From<ApiError<T>> for Reply
where
    T: ErrorCode + Error,
{
    fn from(ApiError(err): ApiError<T>) -> Self {
        let code = err.code();
        let message = match code {
            500 => format!("InternalServerError: {}", err),
            _ => err.to_string(),
        };

        Self { code, message, data: None }
    }
}

impl<T> IntoResponse for ApiError<T>
where
    T: ErrorCode + Error,
{
    fn into_response(self) -> Response {
        let reply = Reply::<()>::from(self);
        tracing::error!("ErrorResponse: {:?}", reply);
        Json(reply).into_response()
    }
}
