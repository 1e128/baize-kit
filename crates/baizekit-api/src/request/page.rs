use baizekit_serde::num::de_str_or_num_to_u64;
use serde::{Deserialize, Serialize};

/// 分页请求参数
/// 
/// 用于处理分页查询的请求参数，支持字符串或数字类型的页码和大小。
/// 
/// # 字段
/// 
/// - `current`: 当前页码，从 1 开始
/// - `size`: 每页大小
/// 
/// # 默认值
/// 
/// - 默认页码：1
/// - 默认每页大小：10
/// 
/// # 示例
/// ```rust
/// use baizekit_api::request::PageRequest;
/// 
/// // 使用默认值
/// let page = PageRequest::default();
/// assert_eq!(page.current, 1);
/// assert_eq!(page.size, 10);
/// 
/// // 自定义分页参数
/// let json = r#"{"current": "2", "size": 20}"#;
/// let page: PageRequest = serde_json::from_str(json).unwrap();
/// assert_eq!(page.current, 2);
/// assert_eq!(page.size, 20);
/// ```
#[derive(Copy, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct PageRequest {
    /// 请求的页码，从 1 开始
    #[serde(deserialize_with = "de_str_or_num_to_u64")]
    pub current: u64,
    /// 每页大小
    #[serde(deserialize_with = "de_str_or_num_to_u64")]
    pub size: u64,
}

impl Default for PageRequest {
    fn default() -> Self { 
        PageRequest { 
            current: 1, 
            size: 10 
        } 
    }
}

/// 获取默认页码
/// 
/// 返回默认的页码值（1）
#[inline]
pub fn default_page() -> u64 { 1 }

/// 获取默认每页大小
/// 
/// 返回默认的每页大小值（10）
#[inline]
pub fn default_page_size() -> u64 { 10 }
