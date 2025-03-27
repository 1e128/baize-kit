use std::future::Future;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode, Uri};
use axum::response::Response;

use crate::prelude::AdminPrincipal;

pub trait AuthenticationTrait {
    fn has_permission(
        &self,
        user_id: i64,
        method: Method,
        path: String,
    ) -> impl Future<Output = Result<bool, String>> + Send;
}

pub struct AuthorizedAdminPrincipal(pub AdminPrincipal);

impl<S> FromRequestParts<S> for AuthorizedAdminPrincipal
where
    S: AuthenticationTrait + Send + Sync,
{
    type Rejection = Response<Body>;

    fn from_request_parts(parts: &mut Parts, state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let method = parts.method.clone();

            let mut parts_clone = parts.clone();
            let req_url = match Uri::from_request_parts(&mut parts_clone, state).await {
                Ok(inner) => inner.path().to_string(),
                Err(err) => {
                    tracing::error!("解析路径失败. error: {}", err);
                    return Err(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(err.to_string()))
                        .unwrap());
                }
            };
            let principal = AdminPrincipal::from_request_parts(parts, state).await?;

            tracing::info!("检查用户权限: {}, {}, {}", principal.admin_id, method, req_url);

            match state.has_permission(principal.admin_id, method.clone(), req_url.clone()).await {
                Ok(inner) if inner => Ok(Self(principal)),
                Ok(_) => {
                    tracing::info!("用户(id: {})没有权限访问 {} - {}", principal.admin_id, method, req_url);
                    Err(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Forbidden"))
                        .unwrap())
                }
                Err(e) => {
                    tracing::error!("服务内部错误. error: {}", e);
                    Err(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(e))
                        .unwrap())
                }
            }
        }
    }
}
