use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use anyhow::Result;
use log::debug;
use regex::Regex;

static MOD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(pub\s+)?mod\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*);").unwrap());

/// 更新模块声明
pub fn update_mod_rs(mod_dir: PathBuf) -> Result<()> {
    let mod_rs_path = mod_dir.join("mod.rs");
    if !mod_rs_path.exists() {
        anyhow::bail!("目标目录{}不存在mod.rs ", mod_dir.display())
    }

    // 1. 读取已有 mod.rs 的行（用于查重）
    let file = std::fs::File::open(&mod_rs_path).expect("无法读取 mod.rs");
    let reader = BufReader::new(file);

    let mut existing_mods = HashSet::new();
    for line in reader.lines() {
        let line = line?;
        if let Some(mod_name) = MOD_REGEX.captures(&line) {
            existing_mods.insert(mod_name["name"].to_string());
        }
    }

    // 2. 收集当前目录下的所有mod
    let mut new_mods = vec![];
    for entry in std::fs::read_dir(&mod_dir).expect("读取目录失败") {
        let path = entry?.path();

        // 忽略 mod.rs 自身
        if path.file_name().map_or(false, |v| v == "mod.rs") {
            continue;
        }

        // 普通 .rs 文件
        if path.is_file()
            && path.extension().map_or(false, |e| e == "rs")
            && let Some(mod_name) = path.file_stem().map(|v| v.to_string_lossy().to_string())
            && !existing_mods.contains(&mod_name)
        {
            new_mods.push(format!("pub mod {};", mod_name));
        }

        // 目录形式（包含 mod.rs）
        if path.is_dir()
            && path.join("mod.rs").exists()
            && let Some(mod_name) = path.file_name().map(|v| v.to_string_lossy().to_string())
            && !existing_mods.contains(&mod_name)
        {
            new_mods.push(format!("pub mod {};", mod_name));
        }
    }

    if new_mods.is_empty() {
        debug!("没有需要新增的 mod");
        return Ok(());
    }

    // 3. 添加新mod到mod.rs中
    add_mods_into_mod_rs(&mod_rs_path, new_mods)?;
    Ok(())
}

/// 添加声明到mod.rs中
fn add_mods_into_mod_rs(mod_rs_path: &Path, new_mods: Vec<String>) -> Result<()> {
    let mod_content = std::fs::read_to_string(&mod_rs_path)?;
    let mut updated_mod_content = mod_content.clone();

    // 创建一个备份文件
    let mod_backup_filepath = mod_rs_path.with_extension("rs.bak");
    std::fs::copy(&mod_rs_path, &mod_backup_filepath)?;
    let mut mod_rs_file = std::fs::File::create(&mod_rs_path)?;

    let mods: Vec<_> = MOD_REGEX.captures_iter(&mod_content).collect();
    // 找到最后一句mod声明
    let mods_end = mods
        .last()
        .map(|last_match| last_match.get(0).map(|c| c.end() + 1))
        .flatten()
        .unwrap_or_else(|| mod_content.len());

    for new_mod in new_mods {
        updated_mod_content.insert_str(mods_end, &format!("{}\n", new_mod));
    }

    mod_rs_file.write_all(updated_mod_content.as_bytes())?;
    std::fs::remove_file(&mod_backup_filepath)?;

    // 格式化代码
    let _ = Command::new("rustfmt")
        .arg("+nightly")
        .arg(mod_rs_path.to_string_lossy().to_string())
        .args(vec!["--edition", "2024"])
        .status();
    Ok(())
}
