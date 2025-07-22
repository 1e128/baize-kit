use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
pub use tracing::Level;

use crate::format::LogFormat;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct LogConfig {
    /// 日志格式
    pub format: LogFormat,
    /// 日志等级
    #[serde(with = "::serde_with::As::<DisplayFromStr>")]
    pub level: Level,
    /// 是否显示 ANSI 颜色
    pub ansi: bool,
    /// 是否显示文件名
    pub with_filename: bool,
    /// 是否显示行号
    pub with_line_number: bool,
    /// 是否显示时间
    pub with_time: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            format: LogFormat::Compact,
            level: Level::INFO,
            with_filename: true,
            with_line_number: true,
            ansi: true,
            with_time: true,
        }
    }
}
