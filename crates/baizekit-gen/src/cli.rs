use clap::{Parser, Subcommand};

use crate::domain::GenerateDomainCommand;
use crate::repository::GenerateRepositoryCommand;

#[derive(Parser, Debug)]
#[command(
    version,
    author,
    help_template = r#"{before-help}{name} {version}
{about-with-newline}

{usage-heading} {usage}

{all-args}{after-help}

AUTHORS:
    {author}
"#,
    about = r#"好的，收到"#
)]
pub struct Cli {
    #[arg(global = true, short, long, help = "Show debug messages")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    #[command(about = "生成 Entity、Repository 文件")]
    Domain(GenerateDomainCommand),
    #[command(visible_alias = "repo", about = "生成 RepositoryImpl 文件")]
    Repository(GenerateRepositoryCommand),
}
