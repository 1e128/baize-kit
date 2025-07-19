use std::path::PathBuf;

use cargo_generate::{generate, GenerateArgs};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use clap::Args;
use log::info;

use crate::config::{config_file_path, BaizeConfig, BaizeTemplate};

#[derive(Clone, Debug, Args)]
#[clap(arg_required_else_help(true))]
pub struct GenerateServiceCommand {
    #[arg(short, long = "service_name")]
    pub service_name: String,

    #[arg(short, long, help = "目标路径, 如果不指定，则从配置中读取")]
    pub destination: Option<Utf8PathBuf>,

    #[arg(long, short, number_of_values = 1, value_parser, help = "模板参数，例如：--config key=value")]
    pub template_values: Vec<String>,
}

impl GenerateServiceCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps() // 可选，表示不获取依赖项信息
            .exec()
            .expect("无法获取 cargo metadata");
        info!("WorkspaceRoot: {:#?}", metadata.workspace_root);

        let config_file = config_file_path(metadata.workspace_root.as_std_path());
        let config_str = std::fs::read_to_string(config_file)?;
        let config: BaizeConfig = toml::from_str(&config_str)?;

        let service_template = config.templates.get("crates").unwrap(); // todo: z

        let target_path = self
            .destination
            .clone()
            .unwrap_or_else(|| metadata.workspace_root.join(service_template.config.destination.clone()));
        self.run_cargo_generate(service_template, target_path.into_std_path_buf());
        Ok(())
    }

    fn run_cargo_generate(&self, template: &BaizeTemplate, target_path: PathBuf) {
        let mut args = GenerateArgs::default();
        args.template_path.path = Some(template.path.to_string_lossy().to_string());
        args.init = template.config.init;
        args.name = Some(self.service_name.clone());
        args.destination = Some(target_path);
        args.define = self.template_values.iter().map(ToString::to_string).collect::<Vec<_>>();

        let _ = generate(args)
            .inspect(|path| info!("Generated: {}", path.display()))
            .inspect_err(|err| info!("Generate Failed. error: {}", err));
    }
}
