use chrono::FixedOffset;

const DATETIME_WITH_SECONDS_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const DATETIME_WITH_MILLISECONDS_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";
const UTC8_OFFSET: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

/// 将 时区为UTC的时间字符串 反序列化为 DateTime<Utc>
/// 将 DateTime<Utc> 序列化为 时区为UTC的时间字符串
pub mod datetime_utc_with_seconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    use super::DATETIME_WITH_SECONDS_FORMAT;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, DATETIME_WITH_SECONDS_FORMAT)
            .map(|dt| dt.and_utc())
            .map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(DATETIME_WITH_SECONDS_FORMAT).to_string();
        serializer.serialize_str(&s)
    }
}

/// 将 时区为UTC8的时间字符串 反序列化为 DateTime<Utc>
/// 将 DateTime<Utc> 序列化成 时区为UTC8的时间字符串
pub mod datetime_utc8_with_seconds {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::datetime::{DATETIME_WITH_SECONDS_FORMAT, UTC8_OFFSET};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // 解析成 UTC+8 的时间
        let dt_with_offset = NaiveDateTime::parse_from_str(&s, DATETIME_WITH_SECONDS_FORMAT)
            .map(|v| v.and_local_timezone(UTC8_OFFSET).single())
            .map_err(Error::custom)?
            .ok_or_else(|| Error::custom(format!("Invalid time: {}", s)))?;

        // 转换为 UTC 时间
        Ok(dt_with_offset.with_timezone(&Utc))
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .with_timezone(&UTC8_OFFSET)
            .format(DATETIME_WITH_SECONDS_FORMAT)
            .to_string();
        serializer.serialize_str(&s)
    }
}

/// 将 时区为UTC的时间字符串 反序列化为 DateTime<Utc>
/// 将 DateTime<Utc> 序列化为 时区为UTC的时间字符串
pub mod datetime_utc_with_milliseconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    use super::DATETIME_WITH_MILLISECONDS_FORMAT;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, DATETIME_WITH_MILLISECONDS_FORMAT)
            .map(|dt| dt.and_utc())
            .map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(DATETIME_WITH_MILLISECONDS_FORMAT).to_string();
        serializer.serialize_str(&s)
    }
}

/// 将 时区为UTC8的时间字符串 反序列化为 DateTime<Utc>
/// 将 DateTime<Utc> 序列化成 时区为UTC8的时间字符串
pub mod datetime_utc8_with_milliseconds {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    use super::{DATETIME_WITH_MILLISECONDS_FORMAT, UTC8_OFFSET};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // 解析为 UTC+8 的时间
        let dt_with_offset = NaiveDateTime::parse_from_str(&s, DATETIME_WITH_MILLISECONDS_FORMAT)
            .map(|v| v.and_local_timezone(UTC8_OFFSET).single())
            .map_err(Error::custom)?
            .ok_or_else(|| Error::custom(format!("Invalid time: {}", s)))?;

        // 转换为 UTC 时间
        Ok(dt_with_offset.with_timezone(&Utc))
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .with_timezone(&UTC8_OFFSET)
            .format(DATETIME_WITH_MILLISECONDS_FORMAT)
            .to_string();
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_tokens, Token};

    use super::datetime_utc8_with_milliseconds;

    // 定义一个用于测试的结构体，包含一个 DateTime<Utc> 字段，并使用自定义的序列化/反序列化方法
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(with = "datetime_utc8_with_milliseconds")]
        dt: DateTime<Utc>,
    }

    // 辅助函数：构建一个特定时间的 DateTime<Utc>
    fn make_datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        millis: u32,
    ) -> DateTime<Utc> {
        let date_str =
            format!("{}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}", year, month, day, hour, minute, second, millis);
        chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap()
            .and_utc()
    }

    #[test]
    fn test_serialize_valid_datetime() {
        // 构造一个合法的时间值
        let dt = make_datetime(2025, 4, 5, 12, 34, 56, 789);
        let test = TestStruct { dt };

        // 序列化成字符串
        let serialized = serde_json::to_string(&test).unwrap();
        println!("{}, {}", dt, serialized);
        assert!(serialized.contains("2025-04-05 12:34:56.789"));
    }

    #[test]
    fn test_deserialize_valid_string() {
        // JSON 中的时间字符串
        let json_data = r#"{"dt":"2025-04-05 12:34:56.789"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        let serialized = serde_json::to_string(&deserialized).unwrap();

        let expected_dt = make_datetime(2025, 4, 5, 12, 34, 56, 789);
        println!("deserialized: {:?}, serialized: {:?}, expected_at: {}", deserialized, serialized, expected_dt);
        assert_eq!(deserialized.dt, expected_dt);
    }

    #[test]
    fn test_deserialize_invalid_format() {
        // 错误格式的时间字符串
        let json_data = r#"{"dt":"2025/04/05 12:34:56"}"#; // 使用斜杠且缺少毫秒
        let result: Result<TestStruct, _> = serde_json::from_str(json_data);

        // 应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        // 测试完整的序列化和反序列化流程是否保持一致性
        let dt = make_datetime(2025, 1, 1, 0, 0, 0, 0);
        let test = TestStruct { dt };

        assert_tokens(
            &test,
            &[
                Token::Struct { name: "TestStruct", len: 1 },
                Token::Str("dt"),
                Token::Str("2025-01-01 00:00:00.000"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_deserialize_truncated_millis() {
        // 毫秒部分不足三位的情况，例如 ".12"
        let json_data = r#"{"dt":"2025-04-05 12:34:56.12"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json_data);

        // chrono 的 parse_from_str 允许这种情况，会自动补零
        let expected_dt = make_datetime(2025, 4, 5, 12, 34, 56, 120); // .12 => .120
        if let Ok(deserialized) = result {
            assert_eq!(deserialized.dt, expected_dt);
        } else {
            panic!("Failed to parse truncated milliseconds");
        }
    }

    #[test]
    fn test_deserialize_extra_millis() {
        // 毫秒部分超过三位的情况，例如 ".1234"
        let json_data = r#"{"dt":"2025-04-05 12:34:56.1234"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json_data);

        // chrono 的 parse_from_str 会忽略多余的部分
        let expected_dt = make_datetime(2025, 4, 5, 12, 34, 56, 123); // .1234 => .123
        if let Ok(deserialized) = result {
            assert_eq!(deserialized.dt, expected_dt);
        } else {
            panic!("Failed to parse extra milliseconds");
        }
    }
}
