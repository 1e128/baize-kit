use baizekit::app::anyhow;
use baizekit::app::anyhow::Context;
use baizekit::app::async_trait::async_trait;
use baizekit_app::application::{ComponentContext, ComponentKey, InitStrategy};
use baizekit_app::component::Component;
use clap::Subcommand;
use serde::Deserialize;
use std::any::{TypeId};
use std::pin::Pin;
use tracing::info;

// 定义子命令
#[derive(Debug, Subcommand, Clone)]
enum Commands {
    /// 运行 Axum 服务器
    Serve,
    /// 打印 Axum 服务器端口
    PrintPort,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use baizekit::app::new_app;
    use baizekit::component::{DbComponent, LogComponent};

    new_app!(Commands) // 修改为带有 Commands 子命令
        .register_component_factory(None::<&str>, LogComponent::new)
        .register_component_factory(None::<&str>, DbComponent::new)
        .register_component_factory(None::<&str>, AxumComponent::new) // 注册 AxumComponent
        .set_default_handler(|_app| {
            let fut = Box::pin(async {
                info!("Default handler executed.");
                Ok(())
            });
            let inits = vec![ComponentKey { type_id: TypeId::of::<LogComponent>(), label: "default".to_string() }];
            (InitStrategy::Only(inits), fut)
        })
        .register_command_handler(|command, app| {
            // 注册命令处理器
            let fut: Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> = Box::pin(async move {
                match command {
                    Commands::Serve => {
                        info!("Serving Axum application...");
                        // 在这里可以启动 Axum 服务器
                        app.with_component::<AxumComponent, _>(None, |axum_comp| {
                            info!("Axum server will run on port: {}", axum_comp.port());
                        })
                        .await
                        .context("AxumComponent not found")?;
                        // 模拟服务器运行，等待关闭信号
                        info!("Axum server shutting down.");
                    }
                    Commands::PrintPort => {
                        //这里让app不要等待关闭信号
                        app.set_wait_signal(false);
                        app.with_component::<AxumComponent, _>(None, |axum_comp| {
                            println!("Axum server port: {}", axum_comp.port());
                        })
                        .await
                        .context("AxumComponent not found")?;
                    }
                }
                Ok(())
            });
            (InitStrategy::All, fut)
        })
        .run()
        .await
}

#[derive(Debug, Deserialize, Clone)]
pub struct AxumConfig {
    pub port: u16,
}

pub struct AxumComponent {
    config: AxumConfig,
}

impl AxumComponent {
    pub fn new<'a>(
        ctx: &'a ComponentContext<'a>,
        label: &str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self>> + Send + 'a>> {
        Box::pin(async move {
            let config: AxumConfig = ctx.config().get("server").context("Failed to get server config")?;
            info!("AxumComponent new with config: {:?}", config);
            Ok(AxumComponent { config })
        })
    }

    pub fn port(&self) -> u16 {
        self.config.port
    }
}

#[async_trait]
impl Component for AxumComponent {}
