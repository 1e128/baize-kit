//! BaizeKit Serde 模块
//! 
//! 该模块提供了用于序列化和反序列化的工具集，包括：
//! - 数字类型转换
//! - 字符串处理
//! - Decimal 类型序列化
//! 
//! # 功能特性
//! 
//! - 支持字符串和数字之间的灵活转换
//! - 处理空字符串和 null 值的智能转换
//! - Decimal 类型与浮点数的序列化转换
//! 
//! # 示例
//! ```rust
//! use baizekit_serde::prelude::*;
//! use serde::{Deserialize, Serialize};
//! 
//! #[derive(Serialize, Deserialize)]
//! struct Example {
//!     #[serde(deserialize_with = "de_str_or_num_to_u64")]
//!     id: u64,
//!     
//!     #[serde(deserialize_with = "de_empty_string_or_null_to_none")]
//!     description: Option<String>,
//! }
//! ```

pub mod dec;    // Decimal 类型序列化模块
pub mod num;    // 数字类型转换模块
pub mod str;    // 字符串处理模块

/// 预导入模块
/// 
/// 包含最常用的序列化和反序列化函数
pub mod prelude {
    pub use crate::dec::*;
    pub use crate::num::*;
    pub use crate::str::*;
}
