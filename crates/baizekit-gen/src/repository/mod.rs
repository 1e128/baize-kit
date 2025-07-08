use clap::Args;
use convert_case::{Case, Casing};
use tera::{Context, Tera};

use crate::utils::render;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct GenerateRepositoryCommand {
    #[arg(required = true, help = "Entity名称")]
    entity_name: String,

    #[arg(short, long, help = "核心包名。存放 entity 的包")]
    core_package_name: Option<String>,

    #[arg(short, long, help = "输出文件目录")]
    output_dir: Option<String>,
}

impl GenerateRepositoryCommand {
    pub fn run(self) {
        let mut tera = Tera::default();
        tera.add_raw_template("repository.rs.tera", include_str!("./template/repository.rs.tera"))
            .unwrap();

        // 构建上下文
        let mut context = Context::new();

        if let Some(core) = &self.core_package_name {
            context.insert("use_domain_entity", &format!("use {}::{}::*", core, self.entity_name));
        }

        context.insert("entity_name", &self.entity_name.to_case(Case::Pascal));

        for tpl_name in tera.get_template_names() {
            match tera.render(tpl_name, &context) {
                Ok(rendered) => {
                    // 输出文件路径：去掉 .tera 后缀
                    let output_filename = format!("{}.rs", self.entity_name);
                    render(output_filename, rendered, &self.output_dir)
                }
                Err(e) => {
                    eprintln!("❌ 渲染模板 {} 失败: {}", tpl_name, e);
                }
            }
        }
    }
}
