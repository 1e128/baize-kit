use baizekit_derive::With;
use globset::{GlobBuilder, GlobMatcher};

use crate::build::generator::CodeGenerator;
use crate::build::handler::HttpHandler;

#[derive(Debug, Default, With)]
pub struct Builder {
    /// 项目目录
    project_path: String,
    /// 目标文件所在目录
    handlers_dir: String,
    /// 目标文件
    file_matcher: Option<GlobMatcher>,
    /// AppState. e.g. crate::setup::state::AppState,
    app_state: String,
    /// 输出文件路径
    output_path: Option<String>,
    /// 输出模块名称
    output_name: String,
}

impl Builder {
    pub fn with_file_glob_matcher(mut self, file_glob: &str) -> Self {
        let matcher = GlobBuilder::new(file_glob)
            .case_insensitive(true)
            .build()
            .unwrap()
            .compile_matcher();
        self.file_matcher = Some(matcher);
        self
    }

    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handlers = HttpHandler::parse(&self.project_path, &self.handlers_dir, &self.file_matcher)?;
        let code = CodeGenerator::new(handlers, self.app_state.clone()).generate_code();

        let out_dir = self.output_path.clone().unwrap_or_else(|| std::env::var("OUT_DIR").unwrap());
        std::fs::write(format!("{}/{}.rs", out_dir, self.output_name), code)?;

        println!("cargo:rerun-if-changed={}", self.handlers_dir);
        Ok(())
    }
}
