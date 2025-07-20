use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_util_schemas::manifest::TomlManifest;
use glob::glob;
use log::warn;

pub struct Workspace {
    manifest: TomlManifest,
    cargo_toml_path: Utf8PathBuf,
}

impl Workspace {
    pub fn try_new(workspace_root: Utf8PathBuf) -> Result<Self> {
        let cargo_toml_path = workspace_root.join("Cargo.toml");

        let content = fs::read_to_string(cargo_toml_path.as_std_path())?;
        let manifest: TomlManifest = toml::from_str(&content)?;
        if manifest.workspace.is_none() {
            bail!("{} is not a workspace project", workspace_root);
        }

        Ok(Self { manifest, cargo_toml_path })
    }

    /// Add a new member to the workspace, if it is not already a member.
    /// The member list will be sorted alphabetically.
    pub fn add_member(&mut self, member: WorkspaceMember) -> Result<()> {
        let Some(workspace) = self.manifest.workspace.as_mut() else {
            bail!("There is no workspace project at {}", self.cargo_toml_path);
        };

        let Some(members) = workspace.members.as_mut() else {
            bail!("There are no workspace members yet defined.");
        };

        if members.contains(&member.name) {
            warn!("Project `{}` is already a member of the workspace", member.name);
            return Ok(());
        }

        members.push(member.name.clone());
        members.sort();

        Ok(())
    }

    /// Save the updated manifest to disk.
    pub fn save(&self) -> Result<()> {
        let new_manifest = toml::to_string_pretty(&self.manifest)?;
        let cargo_toml_path = &self.cargo_toml_path;
        fs::write(cargo_toml_path, new_manifest)?;

        Ok(())
    }
}

pub struct WorkspaceMember {
    name: String,
}

impl WorkspaceMember {
    pub fn try_new(member_path: &Path) -> Result<Self> {
        let cargo_toml_path = member_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
        let manifest: TomlManifest =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;

        let pkg = manifest
            .package()
            .ok_or_else(|| anyhow!("No [package] section found in Cargo.toml at {}", cargo_toml_path.display()))?;

        let name = pkg
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("No `package.name` found in Cargo.toml at {}", cargo_toml_path.display()))?
            .to_string();

        Ok(Self { name })
    }

    /// 匹配所有符合指定 glob 模式的成员
    pub fn try_new_from_glob(pattern: &str) -> Result<Self> {
        for entry in glob(pattern)? {
            let path = entry?;
            if !path.join("Cargo.toml").exists() {
                bail!("{} is not a member of the workspace", path.display());
            }
        }

        Ok(Self { name: pattern.to_string() })
    }
}
