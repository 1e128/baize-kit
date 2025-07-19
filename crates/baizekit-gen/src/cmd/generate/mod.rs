use clap::{Args, Subcommand};

mod entity;
mod service;

#[derive(Debug, Subcommand)]
pub enum GenerateSubCommand {
    Svc(service::GenerateServiceCommand),
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
            GenerateSubCommand::Svc(cmd) => cmd.run(),
            GenerateSubCommand::Entity(cmd) => cmd.run(),
        }
    }
}
