use crate::config::LogConfig;
use crate::format::LogFormat;
use crate::timer::LocalTimer;
use baizekit_app::application::ApplicationInner;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;

pub struct LogComponent {
    #[allow(unused)]
    guard: WorkerGuard,
}

impl LogComponent {
    pub async fn new(inner: Arc<ApplicationInner>, label: String) -> baizekit_app::anyhow::Result<Self> {
        let conf = inner.config().await;
        let conf: LogConfig = conf.get("log")?;

        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

        // 初始化并设置日志格式(定制和筛选日志)
        let sub_builder = tracing_subscriber::fmt()
            .with_ansi(conf.ansi)
            .with_file(conf.with_filename)
            .with_line_number(conf.with_line_number)
            .with_max_level(conf.level)
            .with_timer(LocalTimer)
            .with_writer(non_blocking);

        match conf.format {
            LogFormat::Compact => sub_builder.compact().init(),
            LogFormat::Pretty => sub_builder.pretty().init(),
            LogFormat::Json => sub_builder.json().init(),
        };

        Ok(LogComponent { guard })
    }
}

#[async_trait]
impl Component for LogComponent {}
