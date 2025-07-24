use std::any::Any;
use std::pin::Pin;
use std::future::Future;

use async_trait::async_trait;
use config::Config;

use crate::application::ComponentContext;

// 组件接口
#[async_trait]
pub trait Component: Send + Sync + 'static {
    async fn init(&mut self, _config: &Config, _label: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// 支持类型擦除的DynComponent trait
pub trait DynComponent: Component + Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Component + Any + Send + Sync + 'static> DynComponent for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

// 组件工厂类型定义（内部自动处理Future装箱）
pub type ComponentFactory = Box<
    dyn for<'a> Fn(&'a ComponentContext<'a>) -> Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn DynComponent>>> + Send + 'a>>
    + Send
    + Sync
    + 'static,
>;