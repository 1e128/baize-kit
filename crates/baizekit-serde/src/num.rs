use std::fmt;

use serde::de::Visitor;
use serde::{de, Deserializer};

/// 将字符串或数字反序列化为 u64 类型
///
/// 该函数支持以下输入格式：
/// - 数字类型（如：123）
/// - 字符串类型（如："123"）
///
/// # 参数
///
/// - `deserializer`: 反序列化器
///
/// # 返回值
///
/// - `Result<u64, D::Error>`: 成功时返回解析后的 u64 值，失败时返回反序列化错误
///
/// # 示例
/// ```rust
/// use baizekit_serde::num::de_str_or_num_to_u64;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "de_str_or_num_to_u64")]
///     id: u64,
/// }
///
/// // 可以处理数字
/// let json = r#"{"id": 123}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert_eq!(example.id, 123);
///
/// // 也可以处理字符串
/// let json = r#"{"id": "123"}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert_eq!(example.id, 123);
/// ```
pub fn de_str_or_num_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrU64Visitor;

    impl<'de> Visitor<'de> for StringOrU64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result { formatter.write_str("a string or a u64") }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(de::Error::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(StringOrU64Visitor)
}
