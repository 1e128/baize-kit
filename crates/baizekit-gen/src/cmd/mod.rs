use clap::Parser;

mod generate;
mod init;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// 初始化模板配置
    Init(init::InitCommand),
    /// 生成代码
    Generate(generate::Generate),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::Init(cmd) => cmd.run(),
            Commands::Generate(cmd) => cmd.run(),
        }
    }
}
