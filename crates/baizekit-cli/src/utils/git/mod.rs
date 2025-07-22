//! 这个mod的代码基本来自于: https://github.com/cargo-generate/cargo-generate/tree/main/src/git

use std::fs::{read_to_string, remove_dir_all, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;
use git2::{Repository, RepositoryInitOptions};
use tempfile::TempDir;

mod clone_tool;
mod gitconfig;
mod utils;

pub fn init(project_dir: &Path, branch: Option<&str>, force: bool) -> Result<Repository> {
    Repository::discover(project_dir).map_or_else(
        |_| just_init(project_dir, branch),
        |repo| {
            if force { Repository::open(project_dir).or_else(|_| just_init(project_dir, branch)) } else { Ok(repo) }
        },
    )
}

fn just_init(project_dir: &Path, branch: Option<&str>) -> Result<Repository> {
    let mut opts = RepositoryInitOptions::new();
    opts.bare(false);
    if let Some(branch) = branch {
        opts.initial_head(branch);
    }
    Repository::init_opts(project_dir, &opts).map_err(Into::into)
}

/// 删除 .git 目录
pub fn remove_history(project_dir: &Path) -> Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        remove_dir_all(&git_dir)?;
    }
    Ok(())
}

/// clone git repository into temp using libgit2
pub fn clone_git_template_into_temp(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
    gitconfig: Option<&Path>,
    skip_submodules: bool,
) -> Result<TempDir> {
    let git_clone_dir = tempfile::Builder::new().prefix("cargo-generate").tempdir()?;

    let _repo = clone_tool::RepoCloneBuilder::new(git_url)
        .with_branch(branch)
        .with_ssh_identity(identity)?
        .with_submodules(!skip_submodules)
        .with_gitconfig(gitconfig)?
        .with_destination(git_clone_dir.path())?
        .with_tag(tag)
        .with_revision(revision)
        .build()?
        .do_clone()?;

    Ok(git_clone_dir)
}

/// 添加 .gitignore 条目
pub fn add_gitignore_entry(path: &Path, entry: &str) -> Result<()> {
    let mut entries_exist = false;

    if path.exists() {
        let content = read_to_string(path)?;
        entries_exist = content.lines().any(|line| line.trim() == entry.trim());
    }

    if !entries_exist {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "\n{}", entry)?;
    }

    Ok(())
}
