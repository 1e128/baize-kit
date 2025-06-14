use std::{env, fs};
use clap::Parser;
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database};
use sea_orm_cli::{handle_error, run_generate_command, Commands, GenerateSubcommands};
use sea_orm_migration::MigratorTrait;

use std::path::{Path, PathBuf};

pub fn get_cargo_project_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 获取 CARGO_MANIFEST_DIR 环境变量
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "未设置 CARGO_MANIFEST_DIR 环境变量")?;

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

pub async fn generate_entities(args: &str, migration_tables: Vec<String>, entities_out_path: &Path){
    dotenv().ok();
    let patch_args = ["cli", "generate","entity"];

    let args_str: Vec<&str> = patch_args
        .into_iter()
        .chain(args.split_whitespace())
        .to_owned()
        .collect();
    println!("cli Args: {:?}", args_str);
    let cli = sea_orm_cli::Cli::parse_from(args_str);
    match cli.command {
        Commands::Generate { mut command } => {
            #[allow(irrefutable_let_patterns)]
            if let GenerateSubcommands::Entity {
                ref mut tables,
                ref mut output_dir,
                ..
            } = command {
                *tables = migration_tables;
                *output_dir = entities_out_path.to_str().unwrap_or("").to_owned();
            }
            run_generate_command(command, cli.verbose).await.unwrap_or_else(handle_error);
        }
        _ => {}
    }
}
pub async fn db_migration<M>(migrator: M, args: &str)
where M: MigratorTrait,
{
    dotenv().ok();
    let patch_args = ["cli", "migrate"];

    let args_str: Vec<&str> = patch_args
        .into_iter()
        .chain(args.split_whitespace())
        .to_owned()
        .collect();
    println!("Args as &str: {:?}", args_str);
    // let cli = sea_orm_migration::cli::Cli::parse_from(args_str);

    let Ok(url) = env::var("DATABASE_URL") else {
        panic!("Environment variable 'DATABASE_URL' not set");
    };
    let schema = env::var("DATABASE_SCHEMA").unwrap_or("public".to_owned());

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path(schema)
        .to_owned();

    let db = Database::connect(connect_options)
        .await
        .expect("Fail to acquire database connection");

    let cli = sea_orm_cli::Cli::parse_from(args_str);
    match cli.command {
        Commands::Migrate { command, .. } => {
            sea_orm_migration::cli::run_migrate(migrator, &db, command, cli.verbose).await
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
            baizekit_seaorm::db_migration::<$migrator_type>($migrator_instance, args).await;
        }

        /// 生成实体文件
        ///
        /// # 参数
        /// - `args`: 生成实体时的额外参数
        /// - `migration_tables`: 需要生成实体的表名列表
        /// - `entities_relative_path`: 实体文件相对于项目根目录的路径
        pub async fn run_generate_entities(
            args: &str,
            migration_tables: Vec<String>,
            entities_relative_path: &str,
        ) {
            let Ok(mut out_path) = baizekit_seaorm::get_cargo_project_root() else {
                eprintln!("Failed to get cargo project root");
                return;
            };
            println!("Project root: {:?}", out_path);
            out_path.push(file!());
            out_path.pop();
            if !entities_relative_path.is_empty() {
                out_path.push(entities_relative_path);
            }
            if !out_path.exists() {
                eprintln!("Entities path does not exist: {:?}", out_path);
                return;
            }
            baizekit_seaorm::generate_entities(args, migration_tables, out_path.as_path()).await;
        }
    };
}