use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::MetadataCommand;
use clap::Args;
use log::info;
use toml::Value;

use crate::config::*;
use crate::git;

/// 配置文件管理
#[derive(Clone, Debug, Args)]
pub struct InitCommand {
    #[arg(long, default_value = "https://github.com/WTF-US/baizekit-template.git", help = "模板仓库地址")]
    repo: String,
}

impl InitCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        // 获取当前工作空间根目录
        let workspace_root = MetadataCommand::new()
            .no_deps() // 可选，表示不获取依赖项信息
            .exec()
            .expect("无法获取 cargo metadata")
            .workspace_root;

        // 创建模板配置目录
        let template_dir = template_dir(workspace_root.as_std_path());
        if !template_dir.exists() {
            std::fs::create_dir_all(&template_dir)?;
        }

        // 确保 .gitignore 文件存在并包含模板配置目录
        let gitignore_path = workspace_root.join(".gitignore");
        git::add_gitignore_entry(gitignore_path.as_std_path(), BAIZE_TEMPLATE_DIR)?;

        // 下载模板仓库
        fetch_template_repo(&self.repo, template_dir.as_path())?;

        // 扫描模板目录下的所有模板
        let template_dirs = locate_template_configs(template_dir.as_path())?;

        let templates = template_dirs
            .into_iter()
            .map(|template| parse_baize_template(template_dir.as_path(), template))
            .collect::<anyhow::Result<Vec<BaizeTemplate>>>()?;

        // 生成配置文件
        let config = BaizeConfig { templates: templates.into_iter().map(|t| (t.config.name.clone(), t)).collect() };
        let content = toml::to_string_pretty(&config)?;
        let config_file = config_file_path(workspace_root.as_std_path());
        std::fs::write(&config_file, content)?;
        info!("Config file generated: {}", config_file.display());

        Ok(())
    }
}

/// 从远程仓库克隆最新模板并复制到目标目录
fn fetch_template_repo(repo_url: &str, target_dir: &Path) -> anyhow::Result<PathBuf> {
    let (tmp_path, _branch) = git::clone_git_template_into_temp(repo_url, None, None, None, None, None, false)?;
    git::remove_history(tmp_path.path())?;
    info!("Template cloned to {}, deleting .git history", tmp_path.path().display());

    let repo_name = repo_url
        .rsplit('/')
        .next()
        .and_then(|s| s.strip_suffix(".git"))
        .ok_or_else(|| anyhow::anyhow!("Invalid repo URL"))?;

    let target_repo_dir = target_dir.join(repo_name);
    if target_repo_dir.exists() {
        info!("Removing existing template directory: {}", target_repo_dir.display());
        std::fs::remove_dir_all(&target_repo_dir)?;
    }

    // 拷贝到目标目录
    std::fs::rename(tmp_path, target_repo_dir.clone())?;
    info!("Template moved to {}", target_repo_dir.display());
    Ok(target_repo_dir)
}

/// 解析 cargo-generate.toml 文件, 从中提取出对应的模板配置
fn parse_baize_template(base_dir: &Path, template_sub_folder: PathBuf) -> anyhow::Result<BaizeTemplate> {
    let template_dir = base_dir.join(template_sub_folder);
    let template_config_file = template_dir.clone().join(CARGO_TEMPLATE_CONFIG_FILE_NAME);

    let content = std::fs::read_to_string(&template_config_file).with_context(|| "无法读取模板配置文件")?;
    let value: Value = toml::from_str(&content).with_context(|| "解析 TOML 内容失败")?;
    let baize_value = value.get("baize").with_context(|| "缺少 [baize] 配置块")?;

    let baize_template_config: BaizeTemplateConfig = baize_value
        .clone()
        .try_into()
        .with_context(|| "无法将 [baize] 配置块解析为 BaizeTemplateConfig")?;

    Ok(BaizeTemplate { path: template_dir, config: baize_template_config })
}
