use std::fmt::{Debug, Formatter};

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct Reply<T = ()>
where
    T: serde::Serialize,
{
    /// 响应状态码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> Reply<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self { code: 0, message: "OK".to_string(), data: Some(data) }
    }
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
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

impl<T> Debug for Page<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("total", &self.total)
            .field("page", &self.current)
            .field("size", &self.size)
            .field("data", &self.data.len())
            .finish()
    }
}
