use baizekit::app::anyhow;
use baizekit::app::anyhow::Context;
use baizekit::app::async_trait::async_trait;
use baizekit_app::application::{ApplicationInner, ComponentKey, InitStrategy};
use baizekit_app::component::Component;
use clap::Subcommand;
use serde::Deserialize;
use std::any::TypeId;
use std::sync::Arc;
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
        .register_component_factory(None, LogComponent::new)
        .register_component_factory(None, DbComponent::new)
        .register_component_factory(None, AxumComponent::new) // 注册 AxumComponent
        .set_default_handler(|_app, _factories| {
            let fut = async {
                info!("Default handler executed.");
                Ok(())
            };
            let inits = vec![ComponentKey { type_id: TypeId::of::<LogComponent>(), label: "default".to_string() }];
            (InitStrategy::Only(inits), fut)
        })
        .register_command_handler(|command, app, _factories| {
            // 注册命令处理器
            let fut = async move {
                match command {
                    Commands::Serve => {
                        info!("Serving Axum application...");
                        // 在这里可以启动 Axum 服务器
                        let axum = app.must_get_component::<AxumComponent>(None::<&str>).await?;
                    }
                    Commands::PrintPort => {
                        //这里让app不要等待关闭信号
                        app.set_wait_signal(false);
                    }
                }
                Ok(())
            };
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
    pub async fn new(ctx: Arc<ApplicationInner>, label: String) -> anyhow::Result<Self> {
        let config: AxumConfig = ctx.config().await.get("server").context("Failed to get server config")?;
        info!("AxumComponent new with config: {:?}", config);
        Ok(AxumComponent { config })
    }

    pub fn port(&self) -> u16 {
        self.config.port
    }
}

#[async_trait]
impl Component for AxumComponent {}
