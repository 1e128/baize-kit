use std::fs::{read_to_string, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use git2::Repository;
use tempfile::TempDir;

use super::clone_tool::RepoCloneBuilder;

pub fn tmp_dir() -> std::io::Result<TempDir> {
    tempfile::Builder::new().prefix("cargo-generate").tempdir()
}

/// deals with `~/` and `$HOME/` prefixes
pub fn canonicalize_path(p: impl AsRef<Path>) -> Result<PathBuf> {
    let p = p.as_ref();
    let p = if p.starts_with("~/") {
        home()?.join(p.strip_prefix("~/")?)
    } else if p.starts_with("$HOME/") {
        home()?.join(p.strip_prefix("$HOME/")?)
    } else {
        p.to_path_buf()
    };

    p.canonicalize()
        .with_context(|| format!("path does not exist: {}", p.display()))
}

/// home path wrapper
pub fn home() -> Result<PathBuf> {
    home::home_dir().context("$HOME was not set")
}

// clone git repository into temp using libgit2
pub fn clone_git_template_into_temp(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
    gitconfig: Option<&Path>,
    skip_submodules: bool,
) -> Result<(TempDir, Option<String>)> {
    let git_clone_dir = tmp_dir()?;

    let repo = RepoCloneBuilder::new(git_url)
        .with_branch(branch)
        .with_ssh_identity(identity)?
        .with_submodules(!skip_submodules)
        .with_gitconfig(gitconfig)?
        .with_destination(git_clone_dir.path())?
        .with_tag(tag)
        .with_revision(revision)
        .build()?
        .do_clone()?;

    let branch = get_branch_name_repo(&repo).ok();

    Ok((git_clone_dir, branch))
}

/// thanks to @extrawurst for pointing this out
/// <https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/branch/mod.rs#L38>
fn get_branch_name_repo(repo: &Repository) -> anyhow::Result<String> {
    let iter = repo.branches(None)?;

    for b in iter {
        let b = b?;

        if b.0.is_head() {
            let name = b.0.name()?.unwrap_or("");
            return Ok(name.into());
        }
    }

    anyhow::bail!("A repo has no Head")
}

/// 添加 .gitignore 条目
pub fn add_gitignore_entry(path: &Path, entry: &str) -> anyhow::Result<()> {
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
