use std::error::Error;

use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::response::{ErrorCode, Reply};

/// API 错误包装器
///
/// 用于将实现了 `ErrorCode` 和 `Error` 特征的类型转换为 HTTP 响应。
///
/// # 类型参数
///
/// - `T`: 错误类型，必须同时实现 `ErrorCode` 和 `Error` 特征
///
/// # 示例
/// ```rust
/// use std::error::Error;
/// use std::fmt::{Display, Formatter};
///
/// use axum::response::IntoResponse;
/// use baizekit_api::response::{ApiError, ErrorCode};
///
/// #[derive(Debug)]
/// struct CustomError;
///
/// impl Display for CustomError {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         todo!()
///     }
/// }
///
/// impl Error for CustomError {}
///
/// impl ErrorCode for CustomError {
///     fn code(&self) -> i32 { 400 }
/// }
///
/// // 转换为 HTTP 响应
/// let error = ApiError(CustomError);
/// let response = error.into_response();
/// ```
pub struct ApiError<T>(pub T)
where
    T: ErrorCode + Error;

impl<T> From<T> for ApiError<T>
where
    T: ErrorCode + Error,
{
    fn from(value: T) -> Self { Self(value) }
}

impl<T> IntoResponse for ApiError<T>
where
    T: ErrorCode + Error,
{
    /// 将错误转换为 HTTP 响应
    ///
    /// # 响应格式
    ///
    /// - 状态码：根据 `ErrorCode::code()` 返回
    /// - 消息：
    ///   - 500 错误：包含详细错误信息
    ///   - 其他错误：使用错误描述
    /// - 数据：始终为 null
    ///
    /// # 日志
    ///
    /// 错误会被记录到日志中，使用 `tracing::error!` 宏
    fn into_response(self) -> Response {
        let code = self.0.code();
        let message = match code {
            500 => format!("Internal Server Error: {}", self.0),
            _ => self.0.to_string(),
        };

        let resp: Reply<Option<()>> = Reply { code, message: message.clone(), data: None };
        tracing::error!("Error: {:?}", self.0);
        Json(resp).into_response()
    }
}
