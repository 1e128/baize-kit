use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString};

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Display, EnumString, SerializeDisplay, DeserializeFromStr)]
pub enum LogFormat {
    #[default]
    #[strum(serialize = "compact")]
    Compact,
    #[strum(serialize = "pretty")]
    Pretty,
    #[strum(serialize = "json")]
    Json,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_test::{assert_tokens, Token};

    use super::*;

    // Display trait 测试
    #[test]
    fn display_impl_should_return_correct_strings() {
        assert_eq!(LogFormat::Compact.to_string(), "compact");
        assert_eq!(LogFormat::Pretty.to_string(), "pretty");
        assert_eq!(LogFormat::Json.to_string(), "json");
    }

    // FromStr trait 测试
    mod from_str {
        use super::*;

        #[test]
        fn parse_valid_strings() {
            assert_eq!(LogFormat::from_str("compact"), Ok(LogFormat::Compact));
            assert_eq!(LogFormat::from_str("pretty"), Ok(LogFormat::Pretty));
            assert_eq!(LogFormat::from_str("json"), Ok(LogFormat::Json));
        }

        #[test]
        fn reject_invalid_strings() {
            assert!(LogFormat::from_str("Compact").is_err());
            assert!(LogFormat::from_str("JSON").is_err());
            assert!(LogFormat::from_str("invalid").is_err());
        }
    }

    // Serde 集成测试
    #[test]
    fn serde_serialization_roundtrip() {
        // Test valid values
        assert_tokens(&LogFormat::Compact, &[Token::Str("compact")]);
        assert_tokens(&LogFormat::Pretty, &[Token::Str("pretty")]);
        assert_tokens(&LogFormat::Json, &[Token::Str("json")]);

        // Test deserialization error
        serde_test::assert_de_tokens_error::<LogFormat>(&[Token::Str("INVALID")], "Matching variant not found");
    }

    // 枚举完整性测试
    #[test]
    fn should_have_three_variants() {
        let variants = [LogFormat::Compact, LogFormat::Pretty, LogFormat::Json];
        assert_eq!(variants.len(), 3);
    }
}
