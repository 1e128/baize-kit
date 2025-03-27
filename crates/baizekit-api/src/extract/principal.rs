use std::future::Future;
use std::str::FromStr;
use std::sync::LazyLock;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::header::HeaderName;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use serde::{Deserialize, Serialize};

pub const SYSTEM_TENANT_ID: &str = "SYSTEM_TENANT_ID";
pub const CUSTOM_ADMIN_PRINCIPAL_HEADER: &str = "x-admin-principal";
static CUSTOM_ADMIN_PRINCIPAL_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_str(CUSTOM_ADMIN_PRINCIPAL_HEADER).unwrap());

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminPrincipal {
    pub admin_id: i64,
    pub account: String,
    pub tenant_id: String,
    pub tenant_owner: Option<i64>,
}

impl AdminPrincipal {
    pub fn is_system_admin(&self) -> bool {
        self.tenant_id == SYSTEM_TENANT_ID
    }

    pub fn is_owner(&self) -> bool {
        match self.tenant_owner {
            None => false,
            Some(owner) => self.admin_id == owner,
        }
    }
}

impl<S> FromRequestParts<S> for AdminPrincipal
where
    S: Send + Sync,
{
    type Rejection = Response<Body>;

    fn from_request_parts(parts: &mut Parts, _: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            // 提取自定义的头条目
            if let Some(auth_header) = parts.headers.get(&*CUSTOM_ADMIN_PRINCIPAL_HEADER_NAME) {
                if let Ok(auth_str) = auth_header.to_str() {
                    // 解析JSON字符串成Principal对象
                    if let Ok(principal) = serde_json::from_str::<AdminPrincipal>(auth_str) {
                        tracing::info!("Admin principal: {:?}", principal);
                        return Ok(principal);
                    }
                }
            }

            tracing::info!("Admin principal not found in headers");
            // 如果提取或解析失败，返回 401 Unauthorized 响应
            Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Unauthorized"))
                .unwrap())
        }
    }
}

pub const CUSTOM_PRINCIPAL_HEADER: &str = "x-principal";
#[derive(Debug, Deserialize, Serialize)]
pub struct EndUserPrincipal {
    pub id: i32,
    pub account: String,
    pub tenant_id: String,
}

impl<S> FromRequestParts<S> for EndUserPrincipal
where
    S: Send + Sync,
{
    type Rejection = Response<Body>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let header_name = HeaderName::from_str(CUSTOM_PRINCIPAL_HEADER).unwrap();

        if let Some(auth_header) = parts.headers.get(&header_name) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Ok(principal) = serde_json::from_str::<EndUserPrincipal>(auth_str) {
                    return Ok(principal);
                }
            }
        }

        Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Unauthorized"))
            .unwrap())
    }
}
