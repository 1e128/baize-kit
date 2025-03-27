use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serializer;

/// 将 Decimal 类型序列化为 f64
///
/// 该函数将 Decimal 类型转换为 f64 类型进行序列化。
/// 如果转换失败，将返回一个自定义错误。
///
/// # 参数
///
/// - `decimal`: 要序列化的 Decimal 值
/// - `serializer`: 序列化器
///
/// # 返回值
///
/// - `Result<S::Ok, S::Error>`: 成功时返回序列化结果，失败时返回序列化错误
///
/// # 示例
/// ```rust
/// use baizekit_serde::dec::ser_decimal_as_f64;
/// use rust_decimal::Decimal;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "ser_decimal_as_f64")]
///     amount: Decimal,
/// }
/// ```
pub fn ser_decimal_as_f64<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let float_value = decimal
        .to_f64()
        .ok_or_else(|| serde::ser::Error::custom(format!("Failed to convert Decimal to f64: {:?}", decimal)))?;
    serializer.serialize_f64(float_value)
}

/// 将 Option<Decimal> 类型序列化为 Option<f64>
///
/// 该函数将 Option<Decimal> 类型转换为 Option<f64> 类型进行序列化。
/// 如果 Decimal 转换失败，将使用 0.0 作为默认值。
///
/// # 参数
///
/// - `opt_decimal`: 要序列化的 Option<Decimal> 值
/// - `serializer`: 序列化器
///
/// # 返回值
///
/// - `Result<S::Ok, S::Error>`: 成功时返回序列化结果，失败时返回序列化错误
///
/// # 示例
/// ```rust
/// use baizekit_serde::dec::ser_decimal_opt_as_f64_opt;
/// use rust_decimal::Decimal;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "ser_decimal_opt_as_f64_opt")]
///     amount: Option<Decimal>,
/// }
/// ```
pub fn ser_decimal_opt_as_f64_opt<S>(opt_decimal: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt_decimal {
        Some(decimal) => serializer.serialize_f64(decimal.to_f64().unwrap_or(0.0)),
        None => serializer.serialize_none(),
    }
}
