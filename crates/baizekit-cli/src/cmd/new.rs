use cargo_generate::{generate, GenerateArgs};
use clap::Args;
use log::info;

use crate::utils::git;

#[derive(Clone, Debug, Args)]
pub struct NewAppCommand {
    #[arg(long, default_value = "https://github.com/1e128/baize-template.git", help = "模板仓库地址")]
    pub repo: String,

    #[arg(short, long, default_value = "baize", help = "App名称")]
    pub name: String,
}

impl NewAppCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        info!("current_dir: {}", current_dir.display());

        let mut args = GenerateArgs::default();
        args.template_path.git = Some(self.repo.clone());
        args.template_path.subfolder = Some("app".to_string());
        args.init = false;
        args.name = Some(self.name.clone());

        let _ = generate(args)
            .inspect(|path| info!("Generated: {}", path.display()))
            .inspect_err(|err| info!("Generate Failed. error: {}", err))?;

        let project_path = current_dir.join(self.name.clone());
        git::init(project_path.as_path(), Some("master"), true)?;
        info!("Init git success. {}", project_path.display());
        Ok(())
    }
}
