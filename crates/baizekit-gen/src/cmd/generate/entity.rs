use cargo_generate::{generate, GenerateArgs};
use cargo_metadata::{MetadataCommand, Package};
use clap::Args;
use log::info;

use crate::config::{config_file_path, BaizeConfig, BaizeTemplate};

#[derive(Clone, Debug, Args)]
pub struct GenerateEntityCommand {
    #[arg(short, long = "crate")]
    pub crate_name: String,

    #[arg(short, long, help = "Entity名称. 例如: account")]
    pub entity: String,

    #[arg(short, long, help = "数据库表前缀. 例如: acc")]
    pub db_prefix: String,

    #[arg(long, short, number_of_values = 1, value_parser, help = "模板参数，例如：--config key=value")]
    pub template_values: Vec<String>,
}

impl GenerateEntityCommand {
    pub fn run(mut self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps() // 可选，表示不获取依赖项信息
            .exec()
            .expect("无法获取 cargo metadata");

        let core_crate_name = format!("{}-core", self.crate_name);
        let core_package = metadata
            .packages
            .iter()
            .find(|p| p.name.to_string() == core_crate_name)
            .ok_or_else(|| anyhow::anyhow!("未找到指定 crate 的 core 包"))?;
        info!("core package. name: {}, manifest_path: {}", core_package.name, core_package.manifest_path);

        let sdk_crate_name = format!("{}-sdk", self.crate_name);
        let sdk_package = metadata
            .packages
            .iter()
            .find(|p| p.name.to_string() == sdk_crate_name)
            .ok_or_else(|| anyhow::anyhow!("未找到指定 crate 的 sdk 包"))?;
        info!("sdk package. name: {}, manifest_path: {}", sdk_package.name, sdk_package.manifest_path);

        self.template_values.insert(0, format!("entity={}", self.entity));
        self.template_values.insert(1, format!("db_prefix={}", self.db_prefix));

        let config_file = config_file_path(metadata.workspace_root.as_std_path());
        let config_str = std::fs::read_to_string(config_file)?;
        let config: BaizeConfig = toml::from_str(&config_str)?;

        self.run_cargo_generate(core_package, config.templates.get("db").unwrap()); // todo: z
        self.run_cargo_generate(core_package, config.templates.get("domain").unwrap());
        self.run_cargo_generate(core_package, config.templates.get("service-core").unwrap());
        self.run_cargo_generate(sdk_package, config.templates.get("service-sdk").unwrap());

        Ok(())
    }

    fn run_cargo_generate(&self, package: &Package, template: &BaizeTemplate) {
        let mut args = GenerateArgs::default();
        args.template_path.path = Some(template.path.to_string_lossy().to_string());
        args.init = template.config.init;
        args.name = Some(self.crate_name.clone());
        args.define = self.template_values.iter().map(ToString::to_string).collect::<Vec<_>>();

        let mut target_path = package.manifest_path.clone();
        target_path.pop(); // remove Cargo.toml 
        target_path.push(template.config.destination.clone()); // add destination
        args.destination = Some(target_path.into_std_path_buf()); // set destination into args

        let _ = generate(args)
            .inspect(|path| info!("Generated: {}", path.display()))
            .inspect_err(|err| info!("Generate Failed. error: {}", err));
    }
}
