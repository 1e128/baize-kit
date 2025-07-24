use std::any::Any;
use std::future::Future;
use std::pin::Pin;

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

// 新的组件工厂 trait
#[async_trait]
pub trait AnyComponentFactory: Send + Sync + 'static {
    // This method will create the concrete component and box it as DynComponent
    async fn create(&self, ctx: &ComponentContext<'_>, label: &str) -> anyhow::Result<Box<dyn DynComponent>>;
}

// Type alias for the specific function signature accepted by the factory
pub type ComponentFactoryFn<Comp> =
    for<'a> fn(&'a ComponentContext<'a>, &str) -> Pin<Box<dyn Future<Output = anyhow::Result<Comp>> + Send + 'a>>;

// Generic implementation for any Fn that can act as a factory
#[async_trait]
impl<Comp, F> AnyComponentFactory for F
where
    Comp: DynComponent + 'static,
    F: for<'a> Fn(&'a ComponentContext<'a>, &str) -> Pin<Box<dyn Future<Output = anyhow::Result<Comp>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
{
    async fn create(&self, ctx: &ComponentContext<'_>, label: &str) -> anyhow::Result<Box<dyn DynComponent>> {
        // Call the user-provided factory function
        let comp = self(ctx, label).await?;
        // Convert to dynamic type
        Ok(Box::new(comp) as Box<dyn DynComponent>)
    }
}

// ComponentFactory type alias to be used in App
pub type ComponentFactory = Box<dyn AnyComponentFactory>;
