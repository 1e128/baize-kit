use async_trait::async_trait;
use config::Config;
use std::any::Any;

/// 组件核心 trait，定义组件生命周期
#[async_trait]
pub trait Component: Send + Sync + 'static {
    /// 初始化组件
    async fn init(&mut self, _config: &Config, _label: String) -> anyhow::Result<()> {
        Ok(())
    }

    /// 关闭组件
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 支持动态类型转换的组件 trait
pub trait DynComponent: Component + Any + Send + Sync + 'static {
    /// 转换为 Any 类型，用于向下转型
    fn as_any(&self) -> &dyn Any;

    /// 获取组件类型名称
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Component + Any + Send + Sync + 'static> DynComponent for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
