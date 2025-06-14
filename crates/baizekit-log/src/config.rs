use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
pub use tracing::Level;

pub use crate::format::LogFormat;

fn default_log_level() -> Level {
    Level::INFO
}

fn on() -> bool {
    true
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct LogConfig {
    /// 日志格式
    #[serde(default)]
    pub format: LogFormat,
    /// 日志登记
    #[serde(default = "default_log_level")]
    #[serde(with = "::serde_with::As::<DisplayFromStr>")]
    pub level: Level,
    /// 是否显示文件名
    #[serde(default = "on")]
    pub display_filename: bool,
    /// 是否显示行号
    #[serde(default = "on")]
    pub display_line_number: bool,
    /// 是否显示 ANSI 颜色
    #[serde(default = "on")]
    pub ansi: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            format: LogFormat::default(),
            level: default_log_level(),
            display_filename: true,
            display_line_number: true,
            ansi: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // 辅助函数：创建默认配置
    fn default_config() -> LogConfig {
        LogConfig {
            format: LogFormat::default(),
            level: default_log_level(),
            display_filename: true,
            display_line_number: true,
            ansi: true,
        }
    }

    #[test]
    fn test_deserialize_with_defaults() {
        // TC-001: 空输入应使用全部默认值
        let json = json!({});
        let config: LogConfig = serde_json::from_value(json).unwrap();

        assert_eq!(config.format, LogFormat::default());
        assert_eq!(config.level, default_log_level());
        assert!(config.display_filename);
        assert!(config.display_line_number);
        assert!(config.ansi);
    }

    #[test]
    fn test_full_deserialization() {
        // TC-002: 全字段显式设置
        let json = json!({
            "format": "json",
            "level": "debug",
            "display_filename": false,
            "display_line_number": false,
            "ansi": false
        });

        let config: LogConfig = serde_json::from_value(json).unwrap();

        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, Level::DEBUG);
        assert!(!config.display_filename);
        assert!(!config.display_line_number);
        assert!(!config.ansi);
    }

    #[test]
    fn test_partial_deserialization() {
        // TC-003: 部分字段设置
        let json = json!({
            "level": "WARN",
            "ansi": false
        });

        let config: LogConfig = serde_json::from_value(json).unwrap();
        let default = default_config();

        assert_eq!(config.level, Level::WARN);
        assert!(!config.ansi);
        assert_eq!(config.format, default.format);
        assert_eq!(config.display_filename, default.display_filename);
        assert_eq!(config.display_line_number, default.display_line_number);
    }

    #[test]
    fn test_invalid_level() {
        // TC-005: 非法level值处理
        let json = json!({"level": "INVALID"});
        let result: Result<LogConfig, _> = serde_json::from_value(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_data());
    }

    #[test]
    fn test_clone_and_debug() {
        // 验证Clone和Debug trait
        let config = default_config();
        let cloned = config.clone();

        assert_eq!(config, cloned);
        assert!(!format!("{:?}", config).is_empty());
    }
}
