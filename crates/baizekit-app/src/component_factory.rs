use super::application::{ApplicationInner, ComponentKey};
use super::component::DynComponent;
use async_trait::async_trait;
use std::any::TypeId;
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// 组件工厂 trait，用于创建组件实例
#[async_trait]
pub trait AnyComponentFactory: Send + Sync + 'static {
    /// 创建组件实例
    async fn create(&self, inner: Arc<ApplicationInner>, label: String) -> anyhow::Result<Box<dyn DynComponent>>;
}

/// 组件工厂 trait 的通用实现，适配异步函数
#[async_trait]
impl<Comp, F, Fut> AnyComponentFactory for F
where
    Comp: DynComponent + 'static,
    F: Fn(Arc<ApplicationInner>, String) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<Comp>> + Send + 'static,
{
    async fn create(&self, inner: Arc<ApplicationInner>, label: String) -> anyhow::Result<Box<dyn DynComponent>> {
        let comp = self(inner, label).await?;
        Ok(Box::new(comp) as Box<dyn DynComponent>)
    }
}

/// 组件工厂类型别名
pub type ComponentFactory = Box<dyn AnyComponentFactory>;

/// 组件工厂管理器，负责注册和管理组件工厂
#[derive(Default)]
pub struct ComponentFactoryManager {
    // 存储组件工厂列表和已注册的组件键
    factories: Mutex<(Vec<(ComponentKey, ComponentFactory)>, HashSet<ComponentKey>)>,
}

impl ComponentFactoryManager {
    /// 创建新的组件工厂管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册组件工厂
    pub fn register_component_factory<Comp, F, Fut>(&self, label: Option<impl Into<String>>, factory: F)
    where
        Comp: DynComponent + 'static,
        F: Fn(Arc<ApplicationInner>, String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<Comp>> + Send + 'static,
    {
        let type_id = TypeId::of::<Comp>();
        let label = label.map_or_else(|| "default".to_string(), Into::into);
        let key = ComponentKey { type_id, label };

        // 锁定并检查是否已注册
        let mut factories = self.factories.lock().expect("Failed to lock factories mutex");
        if factories.1.contains(&key) {
            tracing::warn!("组件工厂已注册: {:?}", key);
            return;
        }

        // 包装工厂函数为 ComponentFactory
        let factory_box: ComponentFactory = Box::new(factory);

        // 添加到工厂列表并记录已注册
        factories.0.push((key.clone(), factory_box));
        factories.1.insert(key);
    }

    /// 取出所有工厂并清空内部存储
    pub fn take_factories(&self) -> (Vec<(ComponentKey, ComponentFactory)>, HashSet<ComponentKey>) {
        let mut factories = self.factories.lock().expect("Failed to lock factories mutex");
        (std::mem::take(&mut factories.0), std::mem::take(&mut factories.1))
    }

    /// 检查组件是否已注册
    pub fn is_registered(&self, key: &ComponentKey) -> bool {
        let factories = self.factories.lock().expect("Failed to lock factories mutex");
        factories.1.contains(key)
    }

    /// 获取所有已注册组件的键
    pub fn get_registered_keys(&self) -> Vec<ComponentKey> {
        let factories = self.factories.lock().expect("Failed to lock factories mutex");
        factories.1.iter().cloned().collect()
    }
}
