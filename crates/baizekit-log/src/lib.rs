use std::sync::OnceLock;

pub use tracing;
use tracing_appender::non_blocking::WorkerGuard;

pub use self::config::*;
pub use self::format::*;
use crate::timer::LocalTimer;

mod config;
mod format;
mod timer;

static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init(conf: &LogConfig) {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    // 初始化并设置日志格式(定制和筛选日志)
    let sub_builder = tracing_subscriber::fmt()
        .with_max_level(conf.level)
        .with_file(conf.display_filename)
        .with_line_number(conf.display_line_number) // 写入标准输出
        .with_ansi(conf.ansi) // 关掉ansi的颜色输出功能
        .with_timer(LocalTimer)
        .with_writer(non_blocking);

    match conf.format {
        LogFormat::Compact => sub_builder.compact().init(),
        LogFormat::Pretty => sub_builder.pretty().init(),
        LogFormat::Json => sub_builder.json().init(),
    };

    TRACING_APPENDER_GUARD.set(guard).expect("Failed to set tracing appender guard");
}
