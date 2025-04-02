use chrono::Local;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

// 格式化日志的输出时间格式
pub struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        let formatted_date = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let fraction_seconds = now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        let formatted_fraction_seconds = format!("{:.6}", fraction_seconds).trim_start_matches('0').to_string();
        write!(w, "{}{}", formatted_date, formatted_fraction_seconds)
    }
}
