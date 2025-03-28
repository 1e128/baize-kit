//! BaizeKit API 模块
//!
//! 该模块提供了用于构建 Web API 的工具集，包括：
//! - 请求处理：分页、参数验证等
//! - 响应封装：统一响应格式、错误处理等
//! - 数据提取：请求参数提取、认证信息等
//!
//! # 功能特性
//!
//! - 统一的请求和响应格式
//! - 内置的分页支持
//! - 完善的错误处理机制
//! - 支持 OpenAPI 文档生成
//!
//! # 示例
//! ```rust
//! use baizekit_api::prelude::*;
//! use axum::extract::Json;
//!
//! async fn handler(page: PageRequest) -> Result<ApiOK<Page<()>>, ApiError<()>> {
//!     // 处理请求
//!     let data = Page {
//!         total: 100,
//!         current: page.current,
//!         size: page.size,
//!         data: vec![],
//!     };
//!     Ok(ApiOK::with_data(data))
//! }
//! ```

pub mod extract; // 数据提取模块，用于从请求中提取参数和认证信息
pub mod request; // 请求处理模块，包含分页等通用请求结构
pub mod response; // 响应封装模块，包含统一响应格式和错误处理

/// 预导入模块
///
/// 包含最常用的类型和函数，方便快速使用
pub mod prelude {
    pub use crate::extract::principal::*;
    pub use crate::request::*;
    pub use crate::response::*;
}
