use std::fmt;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

/// 将空字符串或 null 值反序列化为 Option<String>
///
/// 该函数会将以下输入转换为 None：
/// - null 值
/// - 空字符串 ("")
/// - 其他情况保持为 Some(String)
///
/// # 参数
///
/// - `deserializer`: 反序列化器
///
/// # 返回值
///
/// - `Result<Option<String>, D::Error>`: 成功时返回解析后的 Option<String>，失败时返回反序列化错误
///
/// # 示例
/// ```rust
/// use baizekit_serde::str::de_empty_string_or_null_to_none;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "de_empty_string_or_null_to_none")]
///     description: Option<String>,
/// }
///
/// // null 值会被转换为 None
/// let json = r#"{"description": null}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert!(example.description.is_none());
///
/// // 空字符串会被转换为 None
/// let json = r#"{"description": ""}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert!(example.description.is_none());
///
/// // 非空字符串会被转换为 Some
/// let json = r#"{"description": "Hello"}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert_eq!(example.description, Some("Hello".to_string()));
/// ```
pub fn de_empty_string_or_null_to_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNull;

    impl<'de> Visitor<'de> for StringOrNull {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result { write!(formatter, "a string or null") }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // 这里会调用实际的 String 反序列化函数
            let value: Option<String> = Option::deserialize(deserializer)?;
            match value {
                Some(ref v) if v.is_empty() => Ok(None), // 空字符串处理为 None
                other => Ok(other),                      // 非空字符串保持为 Some
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                Ok(None) // 空字符串解析为 None
            } else {
                Ok(Some(value.to_string())) // 非空字符串解析为 Some
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None) // null 解析为 None
        }
    }

    deserializer.deserialize_option(StringOrNull)
}
