use clap::Parser;

mod generate;
mod init;
mod new;

mod style {
    use anstyle::*;
    use clap::builder::Styles;

    const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

    pub const STYLES: Styles = {
        Styles::styled()
            .header(HEADER)
            .usage(USAGE)
            .literal(LITERAL)
            .placeholder(PLACEHOLDER)
            .error(ERROR)
            .valid(VALID)
            .invalid(INVALID)
            .error(ERROR)
    };
}

#[derive(Debug, Parser)]
#[command(author, version, about, styles(style::STYLES))]
pub enum Commands {
    /// 新建一个App项目
    New(new::NewAppCommand),
    /// 初始化模板配置
    Init(init::InitCommand),
    /// 生成代码. [alias: gen]
    #[command(alias = "gen")]
    Generate(generate::Generate),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::New(cmd) => cmd.run(),
            Commands::Init(cmd) => cmd.run(),
            Commands::Generate(cmd) => cmd.run(),
        }
    }
}
