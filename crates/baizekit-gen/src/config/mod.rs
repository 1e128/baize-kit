use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::camino::Utf8PathBuf;

pub const CARGO_TEMPLATE_CONFIG_FILE_NAME: &str = "cargo-generate.toml";

pub const BAIZE_TEMPLATE_DIR: &str = ".template/";
pub const BAIZE_TEMPLATE_CONFIG_FILE_NAME: &str = "config.toml";

pub fn template_dir(base_dir: &Path) -> PathBuf {
    base_dir.join(BAIZE_TEMPLATE_DIR)
}

pub fn config_file_path(base_dir: &Path) -> PathBuf {
    base_dir.join(BAIZE_TEMPLATE_DIR).join(BAIZE_TEMPLATE_CONFIG_FILE_NAME)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BaizeTemplateConfig {
    /// 模板的名称
    pub name: String,
    /// 代码生成的目标目录, 指在哪个目录下生成文件
    pub destination: Utf8PathBuf,
    /// 生成文件时是否要包含文件夹, 还是直接将文件生成在目标目录[destination]下
    pub init: bool,
}

/// 模板
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BaizeTemplate {
    /// 模板所在目录
    pub path: PathBuf,
    /// 模板配置
    pub config: BaizeTemplateConfig,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BaizeConfig {
    /// 模板名 -> 模板
    pub templates: HashMap<String, BaizeTemplate>,
}

pub fn locate_template_configs(base_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::with_capacity(1);

    if base_dir.is_dir() {
        let mut paths_to_search_in = vec![base_dir.to_path_buf()];
        'next_path: while let Some(path) = paths_to_search_in.pop() {
            let mut sub_paths = vec![];
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    sub_paths.push(entry_path);
                } else if entry.file_name() == CARGO_TEMPLATE_CONFIG_FILE_NAME {
                    results.push(path.strip_prefix(base_dir)?.to_path_buf());
                    continue 'next_path;
                }
            }
            paths_to_search_in.append(&mut sub_paths);
        }
    } else {
        results.push(base_dir.to_path_buf());
    }

    results.sort();
    Ok(results)
}
