use clap::{Args, Subcommand};

mod entity;
mod server;

#[derive(Debug, Subcommand)]
pub enum GenerateSubCommand {
    /// 生成服务代码. 包含 core、sdk
    #[command(alias = "svr")]
    Server(server::GenerateServerCommand),
    /// 生成实体代码. 包含 domain、db、service
    Entity(entity::GenerateEntityCommand),
}

#[derive(Debug, Args)]
pub struct Generate {
    #[command(subcommand)]
    pub command: GenerateSubCommand,
}

impl Generate {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            GenerateSubCommand::Server(cmd) => cmd.run(),
            GenerateSubCommand::Entity(cmd) => cmd.run(),
        }
    }
}
