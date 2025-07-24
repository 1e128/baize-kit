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
use std::sync::{Mutex, RwLock as StdRwLock};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::info;

use crate::command::{Cli, EmptyCommand};
use crate::component::{ComponentFactory, DynComponent};
use crate::signal::shutdown_signal;
use crate::version::GLOBAL_VERSION_PRINTER;

// 定义命令处理器类型别名
type CommandHandler<T> = Box<
    dyn Fn(T, &App<T>) -> (InitStrategy, Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>) + Send + Sync + 'static,
>;

// 定义默认处理器类型别名
type DefaultHandler<T> = Box<
    dyn Fn(&App<T>) -> (InitStrategy, Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>) + Send + Sync + 'static,
>;

// 组件唯一标识键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentKey {
    pub type_id: TypeId,
    pub label: String,
}

// 组件工厂上下文
pub struct ComponentContext<'a> {
    pub config: &'a Config,
    pub components: &'a HashMap<ComponentKey, Box<dyn DynComponent>>,
}

impl<'a> ComponentContext<'a> {
    /// 查找组件（返回不可变引用）
    pub fn get_component<C: DynComponent + 'static>(&self, label: Option<&str>) -> Option<&C> {
        let type_id = TypeId::of::<C>();
        let label = label.unwrap_or("default").to_string();
        let key = ComponentKey { type_id, label };

        self.components.get(&key)?.as_any().downcast_ref::<C>()
    }

    pub fn must_get_component<C: DynComponent + 'static>(&self, label: Option<&str>) -> Result<&C> {
        let type_id = TypeId::of::<C>();
        let type_name = std::any::type_name::<C>();
        let label = label.unwrap_or("default").to_string();
        let key = ComponentKey { type_id, label: label.clone() };

        self.components
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("component not found: type_name={:?}, label={}", type_name, label))?
            .as_any()
            .downcast_ref::<C>()
            .ok_or_else(|| anyhow::anyhow!("component type mismatch for label: {}", label))
    }

    /// 通过闭包使用组件不可变引用
    pub fn with_component<C: DynComponent + 'static, R>(
        &self,
        label: Option<&str>,
        f: impl FnOnce(&C) -> R,
    ) -> Option<R> {
        let comp = self.get_component::<C>(label)?;
        Some(f(comp))
    }

    pub fn config(&self) -> &Config {
        self.config
    }
}

// 组件初始化策略
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

// 应用核心结构
pub struct App<T: Subcommand + Clone + 'static = EmptyCommand> {
    default_handler: Mutex<Option<DefaultHandler<T>>>,
    command_handler: Mutex<Option<CommandHandler<T>>>,
    component_factories: Mutex<(Vec<(ComponentKey, ComponentFactory)>, HashMap<ComponentKey, ()>)>,

    components: RwLock<HashMap<ComponentKey, Box<dyn DynComponent>>>,
    config: RwLock<Config>,
    wait_signal: StdRwLock<bool>,
    phantom: PhantomData<T>,
}

impl<T: Subcommand + Clone + 'static> App<T> {
    pub fn new() -> Self {
        Self {
            command_handler: Mutex::new(None),
            component_factories: Mutex::new((Vec::new(), HashMap::new())),
            components: RwLock::new(HashMap::new()),
            config: RwLock::new(Config::default()),
            default_handler: Mutex::new(None),
            wait_signal: StdRwLock::new(true),
            phantom: PhantomData,
        }
    }

    // 注册命令处理器
    pub fn register_command_handler<F>(&self, handler: F) -> &Self
    where
        F: Fn(T, &App<T>) -> (InitStrategy, Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>)
            + Send
            + Sync
            + 'static,
    {
        let mut cmd_handler = self.command_handler.lock().expect("Failed to lock command_handler mutex");
        *cmd_handler = Some(Box::new(handler) as CommandHandler<T>);
        self
    }

    // Simplified register_component_factory
    pub fn register_component_factory<Comp, F>(&self, label: Option<impl Into<String>>, factory: F) -> &Self
    where
        Comp: DynComponent + 'static,
        F: for<'a> Fn(&'a ComponentContext<'a>, &str) -> Pin<Box<dyn Future<Output = Result<Comp>> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        let type_id = TypeId::of::<Comp>();
        let label = label.map_or_else(|| "default".to_string(), |l| l.into());
        let key = ComponentKey { type_id, label: label.clone() };

        let mut factories = self
            .component_factories
            .lock()
            .expect("Failed to lock component_factories mutex");
        if factories.1.contains_key(&key) {
            panic!("component type:{:?}, label:{} already registered!", type_id, key.label);
        }

        // Box the user-provided factory function as AnyComponentFactory
        let factory_boxed: ComponentFactory = Box::new(factory);

        factories.0.push((key.clone(), factory_boxed));
        factories.1.insert(key, ());
        self
    }

    // 设置默认处理器
    pub fn set_default_handler<F>(&self, handler: F) -> &Self
    where
        F: Fn(&App<T>) -> (InitStrategy, Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>) + Send + Sync + 'static,
    {
        let mut def_handler = self.default_handler.lock().expect("Failed to lock default_handler mutex");
        *def_handler = Some(Box::new(handler) as DefaultHandler<T>);
        self
    }

    /// 设置是否等待关闭信号
    pub fn set_wait_signal(&self, wait: bool) -> &Self {
        let mut wait_signal = self.wait_signal.write().expect("Failed to write wait_signal RwLock");
        *wait_signal = wait;
        self
    }

    /// 通过闭包访问组件的不可变引用
    pub async fn with_component<C: DynComponent + 'static, R>(
        &self,
        label: Option<&str>,
        f: impl FnOnce(&C) -> R,
    ) -> Option<R> {
        let type_id = TypeId::of::<C>();
        let label = label.unwrap_or("default").to_string();
        let key = ComponentKey { type_id, label };

        let components = self.components.read().await;
        let comp = components.get(&key)?.as_any().downcast_ref::<C>()?;
        Some(f(comp))
    }

    /// 通过闭包访问组件的可变引用
    pub async fn with_component_mut<C: DynComponent + 'static, R>(
        &self,
        label: Option<&str>,
        f: impl FnOnce(&mut C) -> R,
    ) -> Option<R> {
        let type_id = TypeId::of::<C>();
        let label = label.unwrap_or("default").to_string();
        let key = ComponentKey { type_id, label };

        let mut components = self.components.write().await;
        let comp = components.get_mut(&key)?.as_any_mut().downcast_mut::<C>()?;
        Some(f(comp))
    }

    /// 获取所有已初始化组件的键
    pub async fn get_all_component_keys(&self) -> Vec<ComponentKey> {
        let components = self.components.read().await;
        components.keys().cloned().collect()
    }

    // 应用主入口点
    pub async fn run(&self) -> Result<()> {
        let cli = Cli::<T>::parse();
        //以后换成config component
        self.load_config(&cli.config).await?;

        let (init_strategy, execute_future) = match &cli.command {
            Some(command) => {
                let handler = self
                    .command_handler
                    .lock()
                    .map_err(|e| anyhow::anyhow!("get command handler locked failed: {}", e))?;
                let handler = handler.as_ref().context("handler not registered")?;
                handler(command.clone(), self)
            }
            None => {
                let default_handler = self
                    .default_handler
                    .lock()
                    .map_err(|e| anyhow::anyhow!("get default handler locked failed: {}", e))?;
                let default_handler = default_handler.as_ref();
                if cli.version {
                    self.set_wait_signal(false);

                    let fut: Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> = Box::pin(async {
                        //打印版本信息
                        if let Some(print_version) = GLOBAL_VERSION_PRINTER.get() {
                            print_version();
                        }
                        Ok(())
                    });
                    (InitStrategy::None, fut)
                } else {
                    if let Some(handler) = default_handler {
                        handler(self)
                    } else {
                        let fut: Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> = Box::pin(async { Ok(()) });
                        (InitStrategy::All, fut)
                    }
                }
            }
        };

        self.init_components_with_strategy(init_strategy)
            .await
            .context("init component failed")?;
        execute_future.await?;
        let wait_signal = self
            .wait_signal
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to read wait_signal: {}", e))?;
        if *wait_signal {
            info!("等待ctrl+c信号...");
            shutdown_signal().await;
            info!("收到ctrl+c信号，正在关闭应用...");
        }
        self.shutdown_components().await.context("shutdown component failed")?;
        println!("应用已退出");
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    // 根据策略初始化组件
    async fn init_components_with_strategy(&self, strategy: InitStrategy) -> Result<()> {
        let config = self.config.read().await;
        let mut components = HashMap::new();

        let factories = self
            .component_factories
            .lock()
            .map_err(|e| anyhow::anyhow!("get component factories locked failed: {}", e))?;
        let factories = &factories.0;
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

        // 阶段1: 按注册顺序创建组件实例
        for (key, factory) in &filtered_factories {
            let context = ComponentContext { config: &config, components: &components };
            // Call the create method on the boxed AnyComponentFactory trait object
            let component = factory.create(&context, &key.label).await?;
            let type_name = component.type_name();
            components.insert(key.clone(), component);
            info!(com = type_name, label = key.label, "创建组件成功");
        }

        // 阶段2: 按注册顺序调用init方法
        for (key, _) in &filtered_factories {
            let component = components
                .get_mut(key)
                .context(format!("Component key {:?} not found in components map", key))?;
            let type_name = component.type_name();
            component.init(&config, &key.label).await?;
            info!(com = type_name, label = key.label, "初始化组件成功");
        }

        *self.components.write().await = components;
        Ok(())
    }

    // 关闭所有组件
    async fn shutdown_components(&self) -> Result<()> {
        let mut components = self.components.write().await;
        let mut component_keys: Vec<ComponentKey> = components.keys().cloned().collect();
        component_keys.reverse(); // 反向关闭

        for key in component_keys {
            if let Some(component) = components.get_mut(&key) {
                let type_name = component.type_name();
                component.shutdown().await?;
                info!(com = type_name, label = key.label, "关闭组件成功");
            }
        }
        components.clear();
        Ok(())
    }

    // 加载配置文件
    async fn load_config(&self, config_path: &Option<PathBuf>) -> Result<()> {
        let mut config_builder = Config::builder();

        if let Some(path) = config_path {
            let path = fs::canonicalize(path).context(format!("Failed to canonicalize config path: {:?}", path))?;
            config_builder = config_builder.add_source(File::from(path));
        }

        let config = config_builder.build().context("load config file failed")?;

        *self.config.write().await = config;
        Ok(())
    }
}

impl<T: Subcommand + Clone + 'static> Default for App<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl App<EmptyCommand> {
    pub fn with_empty_command() -> Self {
        Self::new()
    }
}
