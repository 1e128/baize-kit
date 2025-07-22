use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::Parser;
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database};
use sea_orm_cli::{handle_error, run_generate_command, Commands, GenerateSubcommands};
use sea_orm_migration::MigratorTrait;
use tokio::io;
use tokio::io::AsyncBufReadExt;

/// 读取用户输入并检查是否确认(y/Y)
/// 返回 `true` 如果用户输入 'y' 或 'Y'，否则返回 `false`
pub async fn confirm_action(prompt: &str) -> bool {
    // 创建异步标准输入读取器
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();

    // 打印提示信息
    println!("{} (y/N)", prompt);

    // 异步读取用户输入
    match reader.next_line().await {
        Ok(Some(input)) => {
            // 检查输入是否为 'y' 或 'Y'
            input.trim().eq_ignore_ascii_case("y")
        }
        Ok(None) => {
            println!("EOF reached, assuming 'No'");
            false
        }
        Err(e) => {
            println!("Error reading input: {}, assuming 'No'", e);
            false
        }
    }
}

pub fn get_cargo_project_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 获取 CARGO_MANIFEST_DIR 环境变量
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| "未设置 CARGO_MANIFEST_DIR 环境变量")?;

    let mut current_dir = PathBuf::from(&manifest_dir);

    // 循环向上查找，直到根目录
    loop {
        let cargo_toml_path = current_dir.join("Cargo.toml");

        // 检查是否存在 Cargo.toml 文件
        if cargo_toml_path.exists() {
            // 读取文件内容并检查是否包含 [workspace]
            if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
                if content.contains("[workspace]") {
                    return Ok(current_dir);
                }
            }
        }

        // 尝试向上一级目录移动
        let parent = match current_dir.parent() {
            Some(p) => p,
            None => break, // 已经到达根目录
        };

        // 如果无法再向上移动，说明当前目录就是项目根目录
        if parent == current_dir {
            break;
        }

        current_dir = parent.to_path_buf();
    }

    // 如果没有找到工作区，返回原始的 manifest 目录作为项目根目录
    Ok(PathBuf::from(manifest_dir))
}

pub async fn generate_entities(args: &str, migration_tables: Vec<String>, entities_out_path: &Path) {
    dotenv().ok();
    let patch_args = ["cli", "generate", "entity"];

    let args_str: Vec<&str> = patch_args.into_iter().chain(args.split_whitespace()).to_owned().collect();
    println!("cli Args: {:?}", args_str);
    let cli = sea_orm_cli::Cli::parse_from(args_str);
    match cli.command {
        Commands::Generate { mut command } => {
            #[allow(irrefutable_let_patterns)]
            if let GenerateSubcommands::Entity { ref mut tables, ref mut output_dir, .. } = command {
                *tables = migration_tables;
                *output_dir = entities_out_path.to_str().unwrap_or("").to_owned();
            }
            run_generate_command(command, cli.verbose).await.unwrap_or_else(handle_error);
        }
        _ => {}
    }
}
pub async fn db_migration<M>(migrator: M, args: &str)
where
    M: MigratorTrait,
{
    dotenv().ok();
    let patch_args = ["cli", "migrate"];

    let args_str: Vec<&str> = patch_args.into_iter().chain(args.split_whitespace()).to_owned().collect();
    println!("Args as &str: {:?}", args_str);

    let cli = sea_orm_cli::Cli::parse_from(args_str);
    match cli.command {
        Commands::Migrate { command, database_url, database_schema, .. } => {
            let url = match (env::var("DATABASE_URL"), database_url) {
                (_, Some(url)) => url,
                (Ok(url), _) => url,
                _ => panic!("Environment variable 'DATABASE_URL' not set"),
            };
            let schema = match (env::var("DATABASE_SCHEMA"), database_schema) {
                (_, Some(schema)) => schema,
                (Ok(schema), _) => schema,
                _ => "public".to_owned(),
            };
            let prompt = format!("数据库URL:\x1B[31m{}\x1B[0m, Schema:\x1B[31m{}\x1B[0m", url, schema);
            if confirm_action(&prompt).await {
                println!("开始执行数据库迁移...");
            }else{
                println!("取消执行数据库迁移...");
                return 
            }
            let connect_options = ConnectOptions::new(url).set_schema_search_path(schema).to_owned();
            let db = Database::connect(connect_options)
                .await
                .expect("Fail to acquire database connection");
            sea_orm_migration::cli::run_migrate(migrator, &db, command, cli.verbose)
                .await
                .unwrap_or_else(handle_error);
        }
        _ => {}
    }
}

#[macro_export]
macro_rules! define_sea_orm_cli {
    ($migrator_type:ty, $migrator_instance:expr) => {
        /// 运行数据库迁移
        pub async fn run_db_migrations(args: &str) {
            db_migration::<$migrator_type>($migrator_instance, args).await;
        }
    };
}
