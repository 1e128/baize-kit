use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::response::Reply;

/// API 成功响应包装器
/// 
/// 用于将数据转换为标准的成功响应格式。
/// 
/// # 类型参数
/// 
/// - `T`: 响应数据类型，必须实现 `Serialize` 特征
/// 
/// # 示例
/// ```rust
/// use baizekit_api::response::ApiOK;
/// 
/// // 返回数据
/// let response = ApiOK::with_data("Hello");
/// 
/// // 不返回数据
/// let response = ApiOK::without_data();
/// 
/// // 忽略数据
/// let response = ApiOK::ignore_data("Hello");
/// ```
pub struct ApiOK<T>(pub Option<T>)
where
    T: Serialize;

impl<T> IntoResponse for ApiOK<T>
where
    T: Serialize,
{
    /// 将成功响应转换为 HTTP 响应
    /// 
    /// # 响应格式
    /// 
    /// - 状态码：0
    /// - 消息："OK"
    /// - 数据：包含的数据（如果有）
    fn into_response(self) -> Response {
        let resp = Reply {
            code: 0,
            message: String::from("OK"),
            data: self.0,
        };

        Json(resp).into_response()
    }
}

impl<T> ApiOK<T>
where
    T: Serialize,
{
    /// 创建包含数据的成功响应
    /// 
    /// # 参数
    /// 
    /// - `data`: 要包含的数据
    /// 
    /// # 返回值
    /// 
    /// 包含指定数据的成功响应
    #[inline]
    pub fn with_data(data: T) -> Self {
        Self(Some(data))
    }

    /// 创建不包含数据的成功响应
    /// 
    /// # 返回值
    /// 
    /// 不包含数据的成功响应
    #[inline]
    pub fn without_data() -> Self {
        Self(None)
    }

    /// 创建忽略数据的成功响应
    /// 
    /// 用于处理不需要返回数据的场景。
    /// 
    /// # 参数
    /// 
    /// - `_`: 被忽略的数据
    /// 
    /// # 返回值
    /// 
    /// 不包含数据的成功响应
    #[inline]
    pub fn ignore_data(_: T) -> Self {
        Self(None)
    }
}

impl ApiOK<()> {
    /// 创建空的成功响应
    /// 
    /// 用于处理不需要返回任何数据的场景。
    /// 
    /// # 返回值
    /// 
    /// 空的成功响应
    #[inline]
    pub fn empty() -> Self {
        Self(None)
    }
}
