use clap::{Parser, Subcommand};
use std::path::PathBuf;

// CLI命令解析结构
#[derive(Parser, Clone)]
pub struct Cli<T: Subcommand + Clone + 'static> {
    #[arg(long, help = "启用详细输出")]
    pub verbose: bool,

    #[arg(long, help = "配置文件路径")]
    pub config: Option<PathBuf>,

    #[arg(long, help = "日志级别")]
    pub log_level: Option<String>,

    #[arg(long, help = "显示版本信息")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<T>,
}

// 默认空命令
#[derive(Subcommand, Debug, Clone, Default)]
pub enum EmptyCommand {
    #[default]
    #[command(hide = true)]
    None,
}
