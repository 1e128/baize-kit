use super::command::{Cli, EmptyCommand};
use super::component::DynComponent;
use super::component_factory::ComponentFactoryManager;
use super::signal::shutdown_signal;
use super::version::GLOBAL_VERSION_PRINTER;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::{Config, File};
use std::any::TypeId;
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::info;

/// 组件唯一标识键，由类型 ID 和标签组成
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentKey {
    pub type_id: TypeId,
    pub label: String,
}

/// 应用内部状态
#[derive(Default)]
pub struct ApplicationInner {
    config: RwLock<Config>,
    components: RwLock<HashMap<ComponentKey, Arc<dyn DynComponent>>>,
    wait_signal: StdRwLock<bool>,
}

impl ApplicationInner {
    /// 获取组件（返回 Option<Arc<C>>）
    pub async fn get_component<C: DynComponent + 'static>(&self, label: Option<&str>) -> Option<Arc<C>> {
        let type_id = TypeId::of::<C>();
        let label = label.unwrap_or("default").to_string();
        let key = ComponentKey { type_id, label };

        let components = self.components.read().await;
        components
            .get(&key)
            .cloned()
            // 将 Arc<dyn DynComponent> 向下转型为 Arc<C>
            .and_then(|arc| Arc::downcast(arc).ok())
    }

    /// 强制获取组件（返回 Result<Arc<C>>）
    pub async fn must_get_component<C: DynComponent + 'static>(&self, label: Option<&str>) -> Result<Arc<C>> {
        let type_name = std::any::type_name::<C>();
        let label_str = label.unwrap_or("default");

        self.get_component(label).await.ok_or_else(|| {
            anyhow::anyhow!("component not found or type mismatch: type={}, label={}", type_name, label_str)
        })
    }

    /// 获取配置（返回读锁 Guard）
    pub async fn config(&self) -> tokio::sync::RwLockReadGuard<'_, Config> {
        self.config.read().await
    }

    /// 获取配置可变引用（返回写锁 Guard）
    pub async fn config_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, Config> {
        self.config.write().await
    }

    pub fn set_wait_signal(&self, wait: bool) {
        let mut wait_signal = self.wait_signal.write().expect("Failed to write wait_signal RwLock");
        *wait_signal = wait;
    }
}

/// 组件初始化策略
#[derive(Debug, Clone)]
pub enum InitStrategy {
    /// 初始化所有组件
    All,
    /// 不初始化任何组件
    None,
    /// 只初始化指定标签组件
    Only(Vec<ComponentKey>),
    /// 初始化除指定标签外的组件
    Deny(Vec<ComponentKey>),
}

// 定义通用的处理器Future类型别名，简化重复书写
type HandlerFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;

// 命令处理器类型定义（使用别名简化）
type CommandHandler<T> = Box<
    dyn Fn(T, Arc<ApplicationInner>, Arc<ComponentFactoryManager>) -> (InitStrategy, HandlerFuture)
        + Send
        + Sync
        + 'static,
>;

// 默认处理器类型定义（使用别名简化）
type DefaultHandler = Box<
    dyn Fn(Arc<ApplicationInner>, Arc<ComponentFactoryManager>) -> (InitStrategy, HandlerFuture)
        + Send
        + Sync
        + 'static,
>;

/// 应用核心结构
pub struct App<T: Subcommand + Clone + 'static = EmptyCommand> {
    default_handler: StdMutex<Option<DefaultHandler>>,
    command_handler: StdMutex<Option<CommandHandler<T>>>,
    component_factories: Arc<ComponentFactoryManager>,
    inner: Arc<ApplicationInner>,
    phantom: PhantomData<T>,
}

impl<T: Subcommand + Clone + 'static> App<T> {
    /// 创建新的应用实例
    pub fn new() -> Self {
        Self {
            command_handler: StdMutex::new(None),
            component_factories: Arc::new(ComponentFactoryManager::new()),
            inner: Arc::new(ApplicationInner { wait_signal: StdRwLock::new(true), ..ApplicationInner::default() }),
            default_handler: StdMutex::new(None),
            phantom: PhantomData,
        }
    }

    /// 注册命令处理器
    pub fn register_command_handler<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(T, Arc<ApplicationInner>, Arc<ComponentFactoryManager>) -> (InitStrategy, Fut) + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        // 包装处理器，显式转换为动态Future类型
        let wrapped = move |cmd, inner, factories| {
            let (strategy, fut) = handler(cmd, inner, factories);
            (strategy, Box::pin(fut) as HandlerFuture)
        };
        let mut cmd_handler = self.command_handler.lock().expect("Failed to lock command_handler mutex");
        *cmd_handler = Some(Box::new(wrapped) as CommandHandler<T>);
        self
    }

    /// 注册组件工厂
    pub fn register_component_factory<Comp, F, Fut>(&self, label: Option<String>, factory: F) -> &Self
    where
        Comp: DynComponent + 'static,
        F: Fn(Arc<ApplicationInner>, String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Comp>> + Send + 'static,
    {
        self.component_factories.register_component_factory(label, factory);
        self
    }

    /// 设置默认处理器
    pub fn set_default_handler<F, Fut>(&self, handler: F) -> &Self
    where
        F: Fn(Arc<ApplicationInner>, Arc<ComponentFactoryManager>) -> (InitStrategy, Fut) + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        // 包装处理器，显式转换为动态Future类型
        let wrapped = move |inner, factories| {
            let (strategy, fut) = handler(inner, factories);
            (strategy, Box::pin(fut) as HandlerFuture)
        };
        let mut def_handler = self.default_handler.lock().expect("Failed to lock default_handler mutex");
        *def_handler = Some(Box::new(wrapped) as DefaultHandler);
        self
    }

    /// 设置是否等待关闭信号
    pub fn set_wait_signal(&self, wait: bool) -> &Self {
        self.inner.set_wait_signal(wait);
        self
    }

    /// 获取所有已初始化组件的键
    pub async fn get_all_component_keys(&self) -> Vec<ComponentKey> {
        let components = self.inner.components.read().await;
        components.keys().cloned().collect()
    }

    /// 应用主入口点
    pub async fn run(&self) -> Result<()> {
        let cli = Cli::<T>::parse();
        self.load_config(&cli.config).await?;
        let inner_arc = self.inner.clone();
        let factories_arc = self.component_factories.clone();

        // 根据命令或默认情况获取处理策略和执行未来
        let (init_strategy, execute_future) = match &cli.command {
            Some(command) => {
                let handler = self
                    .command_handler
                    .lock()
                    .map_err(|e| anyhow::anyhow!("get command handler lock failed: {}", e))?;
                let handler = handler.as_ref().context("command handler not registered")?;
                handler(command.clone(), inner_arc.clone(), factories_arc.clone())
            }
            None => {
                let default_handler = self
                    .default_handler
                    .lock()
                    .map_err(|e| anyhow::anyhow!("get default handler lock failed: {}", e))?;
                if cli.version {
                    self.set_wait_signal(false);
                    let fut: HandlerFuture = Box::pin(async {
                        if let Some(print_version) = GLOBAL_VERSION_PRINTER.get() {
                            print_version();
                        }
                        Ok(())
                    });
                    (InitStrategy::None, fut)
                } else {
                    default_handler
                        .as_ref()
                        .map(|h| h(inner_arc.clone(), factories_arc.clone()))
                        .unwrap_or_else(|| {
                            let fut: HandlerFuture = Box::pin(async { Ok(()) });
                            (InitStrategy::All, fut)
                        })
                }
            }
        };

        // 初始化组件
        self.init_components_with_strategy(init_strategy)
            .await
            .context("component init failed")?;

        // 执行主逻辑
        execute_future.await?;

        // 处理等待关闭信号
        let wait_signal = self
            .inner
            .wait_signal
            .read()
            .map_err(|e| anyhow::anyhow!("get wait signal lock failed: {}", e))?;
        if *wait_signal {
            info!("等待 ctrl+c 信号...");
            shutdown_signal().await;
            info!("收到 ctrl+c 信号，正在关闭应用...");
        }

        // 关闭组件
        self.shutdown_components().await.context("shutdown component failed.")?;
        println!("应用已退出");
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// 根据策略初始化组件
    async fn init_components_with_strategy(&self, strategy: InitStrategy) -> Result<()> {
        let inner = self.inner.clone();
        // 取出所有工厂并清空内部存储
        let (factories, _) = self.component_factories.take_factories();

        // 过滤需要初始化的工厂
        let filtered_factories: Vec<_> = match strategy {
            InitStrategy::All => factories.iter().collect(),
            InitStrategy::None => Vec::new(),
            InitStrategy::Only(keys_to_init) => {
                factories.iter().filter(|(key, _)| keys_to_init.contains(key)).collect()
            }
            InitStrategy::Deny(keys_to_deny) => {
                factories.iter().filter(|(key, _)| !keys_to_deny.contains(key)).collect()
            }
        };

        // 合并后的组件创建与初始化阶段（按注册顺序）
        for (key, factory) in &filtered_factories {
            // 1. 创建组件实例（此时为可变状态）
            let mut component = factory.create(inner.clone(), key.label.clone()).await?;
            let type_name = component.type_name();

            // 2. 立即初始化组件（使用可变引用）
            let config = inner.config.read().await;
            component.init(&config, key.label.clone()).await?;
            info!(com = type_name, label = &key.label, "组件创建并初始化成功");

            // 3. 转换为Arc并存储（初始化完成后转为不可变共享）
            let arc_component: Arc<dyn DynComponent> = Arc::from(component);
            //因为上面组件init或者factory都有可能会获取component,所以只能在循环中获取写锁
            let mut components = inner.components.write().await;
            components.insert(key.clone(), arc_component);
        }

        Ok(())
    }

    /// 关闭所有组件
    async fn shutdown_components(&self) -> Result<()> {
        let inner = self.inner.clone();
        let mut components = inner.components.write().await;
        let mut component_keys: Vec<ComponentKey> = components.keys().cloned().collect();
        component_keys.reverse(); // 反向关闭（依赖顺序相反）

        for key in component_keys {
            if let Some(component) = components.get(&key) {
                let type_name = component.type_name();
                component.shutdown().await?;
                info!(com = type_name, label = &key.label, "组件关闭成功");
            }
        }

        // 清除组件
        components.clear();
        Ok(())
    }

    /// 加载配置文件
    async fn load_config(&self, config_path: &Option<PathBuf>) -> Result<()> {
        let mut config_builder = Config::builder();
        if let Some(path) = config_path {
            let path = fs::canonicalize(path).context(format!("canonicalize config path failed: {:?}", path))?;
            config_builder = config_builder.add_source(File::from(path));
        }
        let config = config_builder.build().context("config load failed")?;
        *self.inner.config.write().await = config;
        Ok(())
    }
}

impl<T: Subcommand + Clone + 'static> Default for App<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl App<EmptyCommand> {
    /// 创建带有空命令的应用实例
    pub fn with_empty_command() -> Self {
        Self::new()
    }
}
