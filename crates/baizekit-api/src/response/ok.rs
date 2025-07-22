use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::response::Reply;

pub struct ApiOK<T>(pub Option<T>);

impl<T> From<T> for ApiOK<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T> IntoResponse for ApiOK<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let resp = Reply::ok(self.0);
        Json(resp).into_response()
    }
}

impl<T> ApiOK<T> {
    #[inline]
    pub fn with_data(data: T) -> Self {
        Self(Some(data))
    }

    #[inline]
    pub fn without_data() -> Self {
        Self(None)
    }

    #[inline]
    pub fn ignore_data(_: T) -> Self {
        Self(None)
    }
}

impl ApiOK<()> {
    #[inline]
    pub fn empty() -> Self {
        Self(None)
    }
}
