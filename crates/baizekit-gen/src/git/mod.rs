use std::fs::remove_dir_all;
use std::ops::Sub;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use log::warn;
pub use utils::*;

mod clone_tool;
mod gitconfig;
mod utils;

/// remove context of repository by removing `.git` from filesystem
pub fn remove_history(project_dir: &Path) -> Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        let mut attempt = 0_u8;

        loop {
            attempt += 1;
            if let Err(e) = remove_dir_all(&git_dir) {
                if attempt == 5 {
                    anyhow::bail!(e)
                }

                if e.to_string()
                    .contains("The process cannot access the file because it is being used by another process.")
                {
                    let wait_for = Duration::from_secs(2_u64.pow(attempt.sub(1).into()));
                    warn!("Git history cleanup failed with a windows process blocking error. [Retry in {wait_for:?}]");
                    sleep(wait_for);
                } else {
                    anyhow::bail!(e)
                }
            } else {
                return Ok(());
            }
        }
    } else {
        //FIXME should we assume this is expected by caller?
        // panic!("tmp panic");
        Ok(())
    }
}
