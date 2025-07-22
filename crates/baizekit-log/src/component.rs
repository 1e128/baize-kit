use std::future::Future;
use std::pin::Pin;

use baizekit_app::application::ComponentContext;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use baizekit_app::error::{ConfigSnafu, Result, ResultExt};
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::LogConfig;
use crate::format::LogFormat;
use crate::timer::LocalTimer;

pub struct LogComponent {
    #[allow(unused)]
    guard: WorkerGuard,
}

impl LogComponent {
    pub fn new<'a>(ctx: &'a ComponentContext<'a>) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 'a>> {
        Box::pin(async move {
            let conf = ctx.config();
            let conf: LogConfig = conf.get("log").context(ConfigSnafu)?;

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
        })
    }
}

#[async_trait]
impl Component for LogComponent {}
