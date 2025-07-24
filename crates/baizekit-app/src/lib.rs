pub mod application;
pub mod command;
pub mod component;
pub mod signal;
pub mod version;
pub use {anyhow, async_trait, config, vergen_pretty};

#[macro_export]
macro_rules! new_app {
    (@common) => {
        let printer = || {
            use $crate::vergen_pretty::{header, vergen_pretty_env, ConfigBuilder};

            let config = ConfigBuilder::default()
                .env(vergen_pretty_env!())
                .build()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to build version config: {}", e);
                    std::process::exit(1);
                });

            let mut writer = std::io::stdout();
            header(&config, Some(&mut writer))
                .unwrap_or_else(|e| {
                    eprintln!("Failed to print version info: {}", e);
                    std::process::exit(1);
                });
        };

        if let Err(_) = $crate::version::GLOBAL_VERSION_PRINTER.set(printer) {
            eprintln!("Error: Version printer already initialized!");
            eprintln!("Ensure you only call new_app!() once.");
            std::process::exit(1);
        }
    };

    // 模式1：无参数，返回空命令App
    () => {
        {
            $crate::new_app!(@common);
            $crate::application::App::with_empty_command()
        }
    };

    // 模式2：有参数，返回命令App
    ($Command:ty) => {
        {
            $crate::new_app!(@common);
            $crate::application::App::<$Command>::new()
        }
    };
}
