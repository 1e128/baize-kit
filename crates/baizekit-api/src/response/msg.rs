use serde::Serialize;
use utoipa::ToSchema;

/// 统一响应结构
/// 
/// 用于封装所有 API 响应，提供统一的格式和错误处理。
/// 
/// # 字段
/// 
/// - `code`: 响应状态码
///   - 0: 成功
///   - 其他: 错误码
/// - `message`: 响应消息
/// - `data`: 响应数据，可选
/// 
/// # 示例
/// ```rust
/// use baizekit_api::response::Reply;
/// 
/// // 成功响应
/// let success = Reply {
///     code: 0,
///     message: "OK".to_string(),
///     data: Some("Hello"),
/// };
/// 
/// // 错误响应
/// let error = Reply {
///     code: 500,
///     message: "Internal Server Error".to_string(),
///     data: None,
/// };
/// ```
#[derive(Serialize, ToSchema)]
pub struct Reply<T = ()>
where
    T: Serialize,
{
    /// 响应状态码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// 错误码特征
/// 
/// 用于定义错误类型的状态码。
/// 默认返回 500 表示内部服务器错误。
pub trait ErrorCode {
    /// 获取错误码
    /// 
    /// 默认返回 500 表示内部服务器错误
    fn code(&self) -> i32 { 500 }
}

/// 分页数据结构
/// 
/// 用于封装分页查询的结果。
/// 
/// # 字段
/// 
/// - `total`: 总记录数
/// - `current`: 当前页码
/// - `size`: 每页大小
/// - `data`: 当前页数据
/// 
/// # 示例
/// ```rust
/// use baizekit_api::response::Page;
/// 
/// let page = Page {
///     total: 100,
///     current: 1,
///     size: 10,
///     data: vec!["item1", "item2"],
/// };
/// ```
#[derive(Serialize, utoipa::ToSchema)]
pub struct Page<T> {
    /// 总记录数
    pub total: u64,
    /// 当前页码
    pub current: u64,
    /// 每页大小
    pub size: u64,
    /// 当前页数据
    pub data: Vec<T>,
}
