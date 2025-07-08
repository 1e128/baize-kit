use clap::Args;
use convert_case::{Case, Casing};
use tera::{Context, Tera};

use crate::utils::render;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct GenerateDomainCommand {
    #[arg(required = true, help = "Entity名称")]
    entity_name: String,

    #[arg(short, long, help = "输出文件目录")]
    output_dir: Option<String>,
}

impl GenerateDomainCommand {
    pub fn run(self) {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("mod.rs.tera", include_str!("./template/mod.rs.tera")),
            ("entity.rs.tera", include_str!("./template/entity.rs.tera")),
            ("repository.rs.tera", include_str!("./template/repository.rs.tera")),
        ])
        .unwrap();

        // 构建上下文
        let mut context = Context::new();
        context.insert("entity_name", &self.entity_name.to_case(Case::Pascal));

        for tpl_name in tera.get_template_names() {
            match tera.render(tpl_name, &context) {
                Ok(rendered) => {
                    let output_filename = tpl_name.trim_end_matches(".tera");
                    render(output_filename.to_owned(), rendered, &self.output_dir)
                }
                Err(e) => {
                    eprintln!("❌ 渲染模板 {} 失败: {}", tpl_name, e);
                }
            }
        }
    }
}
