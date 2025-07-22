use anyhow::{anyhow, Result};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use dialoguer::Select;  // 用于交互式选择
use dotenvy::dotenv_iter;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use baizekit_seaorm::migration::generate_entities;

mod parse;

use parse::extract_table_names;

#[derive(Parser, Debug)]
pub struct DbCommand {
    /// 指定数据库包
    #[arg(short = 'p', long = "package", help = "identify the db package")]
    pub package: Option<String>,  // 可选参数，用户可手动指定

    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Debug, Subcommand)]
pub enum DbAction {
    #[command(name = "m", about = "migrate operator")]
    Migrate {
        /// 附加参数（放在--之后）
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_terminator = "--",
            value_delimiter = None,
            num_args = 0..,
            help = "additional arguments passed after --"
        )]
        raw_args: Vec<String>,
    },

    #[command(name = "g", about = "generate entities")]
    Generate {
        /// 附加参数（放在--之后）
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_terminator = "--",
            value_delimiter = None,
            num_args = 0..,
            help = "additional arguments passed after --"
        )]
        raw_args: Vec<String>,

        /// 实体输出路径
        #[arg(
            short = 'o',
            long = "output",
            help = "path to output generated entities",
            default_value = "src/_db/entities"
        )]
        output_path: String,
    },
}

impl DbCommand {
    pub fn run(&self) -> Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .map_err(|e| anyhow!("无法获取cargo metadata: {}", e))?;

        // 获取所有以-migrate结尾的包
        let migrate_packages: Vec<String> = metadata
            .packages
            .iter()
            .filter(|p| p.name.ends_with("-migrate"))
            .map(|p| p.name.to_string().clone())
            .collect();

        // 确定要使用的包名
        let package_name = match &self.package {
            Some(pkg) => pkg.clone(),
            None => {
                if migrate_packages.is_empty() {
                    return Err(anyhow!("未找到任何以-migrate结尾的包"));
                } else {
                    // 无论有多少个包，都让用户选择（即使只有一个）
                    println!("请选择要操作的数据库包:");
                    let selection = Select::new()
                        .with_prompt("可用的数据库包")
                        .items(&migrate_packages)
                        .default(0)
                        .interact()
                        .map_err(|e| anyhow!("交互选择失败: {}", e))?;

                    migrate_packages[selection].replace("-migrate", "")  // 提取核心包名
                }
            }
        };

        let migrate_pkg_name = format!("{}-migrate", &package_name);
        let migrate_pkg_name_underscore = migrate_pkg_name.replace('-', "_");

        let migrate_pkg = metadata
            .packages
            .iter()
            .find(|p| p.name.as_str() == migrate_pkg_name.as_str())
            .ok_or_else(|| anyhow!("未找到指定的包: {}", migrate_pkg_name))?;


        match &self.action {
            DbAction::Migrate { raw_args } => {
                println!("Migrating database");
                println!("Package: {}, Raw args: {:?}", package_name, raw_args);

                // 获取migrate包的根目录（目标crate目录）
                let migrate_pkg_root = migrate_pkg
                    .manifest_path
                    .parent()
                    .ok_or_else(|| anyhow!("无法获取migrate包的根目录"))?;

                // 构建src目录路径
                let src_dir = migrate_pkg_root.join("src");
                std::fs::create_dir_all(&src_dir)?;

                // 构建main.rs路径
                let main_rs_path = src_dir.join("main.rs");

                // 检查main.rs是否存在，如果不存在则创建
                if !main_rs_path.exists() {
                    println!("创建main.rs文件: {:?}", main_rs_path);
                    let mut file = File::create(&main_rs_path)?;

                    let main_content = format!(
                        "#[tokio::main]
async fn main() {{
    let args: Vec<String> = std::env::args().collect();

    let args = if args.len() > 1 {{
        args[1..].join(\" \")
    }} else {{
        String::new()
    }};
    {}::run_db_migrations(&args).await;
}}",
                        migrate_pkg_name_underscore
                    );

                    file.write_all(main_content.as_bytes())?;
                }

                // 获取migrate包的绝对路径
                let migrate_pkg_path = migrate_pkg_root
                    .canonicalize()?
                    .to_string_lossy()
                    .to_string();

                // 获取当前目录的.env文件路径（用户执行命令的目录）
                let current_dir = std::env::current_dir()
                    .map_err(|e| anyhow!("无法获取当前目录: {}", e))?;
                let env_file_path = current_dir.join(".env");

                // 加载当前目录的.env文件并收集环境变量
                let mut env_vars = std::env::vars().collect::<Vec<_>>(); // 继承父进程环境变量
                if env_file_path.exists() {
                    println!("加载当前目录的.env文件: {:?}", env_file_path);

                    // 收集.env中的环境变量
                    for item in dotenv_iter()? {
                        let (key, value) = item?;
                        env_vars.push((key, value));
                    }
                } else {
                    println!("当前目录未找到.env文件，使用默认环境变量");
                }

                println!(
                    "在目标crate目录执行: cargo run -p {} 并传递参数: -d {} {:?}",
                    migrate_pkg_name, migrate_pkg_path, raw_args
                );

                // 构建cargo命令：工作目录设为目标crate目录，传递环境变量
                let mut cmd = Command::new("cargo");
                cmd.current_dir(migrate_pkg_root)  // 切换到目标crate目录
                    .args(["run", "-p", &migrate_pkg_name, "--", "-d", &migrate_pkg_path])
                    .args(raw_args)
                    .stdin(Stdio::inherit())   // 支持交互
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());

                // 设置环境变量
                cmd.envs(env_vars);

                // 执行命令并等待结果
                let status = cmd.status()
                    .map_err(|e| anyhow!("无法执行cargo run: {}", e))?;

                if !status.success() {
                    return Err(anyhow!(
                        "cargo run 执行失败，退出码: {}",
                        status.code().unwrap_or(-1)
                    ));
                }

                Ok(())
            }
            DbAction::Generate { raw_args, output_path } => {
                println!("Generating database entities");
                println!("Package: {}, Raw args: {:?}", package_name, raw_args);

                let core_pkg_name = format!("{}-core", &package_name);
                let core_pkg = metadata
                    .packages
                    .iter()
                    .find(|p| p.name.as_str() == core_pkg_name.as_str())
                    .ok_or_else(|| anyhow!("未找到指定的包: {}", core_pkg_name))?;

                let src_dir = migrate_pkg
                    .manifest_path
                    .parent()
                    .ok_or_else(|| anyhow!("无法获取包的目录"))?
                    .join("src");

                if !src_dir.exists() {
                    return Err(anyhow!("源代码目录不存在: {:?}", src_dir));
                }

                let src_dir_path = Path::new(&src_dir);
                let table_infos = extract_table_names(src_dir_path)?;

                println!("\n共找到 {} 个表：", table_infos.len());
                for (i, table_info) in table_infos.iter().enumerate() {
                    println!(
                        "{}. 表名: {} (枚举: {} 模块: {})",
                        i + 1,
                        table_info.table_name,
                        table_info.enum_name,
                        table_info.module
                    );
                }

                let args_str = raw_args.join(" ");

                let migration_tables: Vec<String> = table_infos
                    .iter()
                    .map(|info| info.table_name.clone())
                    .collect();

                let package_dir = core_pkg
                    .manifest_path
                    .parent()
                    .ok_or_else(|| anyhow!("无法获取包的目录"))?;

                let entities_out_path: PathBuf = package_dir.join(output_path).into();

                println!("需要生成的表:{:?}", migration_tables);
                println!("生成实体到路径: {:?}", entities_out_path);

                tokio::runtime::Runtime::new()?
                    .block_on(async {
                        generate_entities(&args_str, migration_tables, &entities_out_path).await;
                    });

                Ok(())
            }
        }
    }
}
