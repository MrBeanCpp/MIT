use crate::{models::Index, utils::util};
use colored::Colorize;
use std::{fs, io, path::PathBuf};

/// 从暂存区&|工作区删除文件
pub fn remove(files: Vec<String>, cached: bool, recursive: bool) -> io::Result<()> {
    util::check_repo_exist();
    let index = Index::get_instance();
    for file in files.iter() {
        let path = PathBuf::from(file);
        if !path.exists() {
            println!("Warning: {} not exist", file.red());
            continue;
        }
        if !index.contains(&path) {
            //不能删除未跟踪的文件
            println!("Warning: {} not tracked", file.red());
            continue;
        }
        if path.is_dir() && !recursive {
            println!("fatal: not removing '{}' recursively without -r", file.bright_blue());
            continue;
        }

        if path.is_dir() {
            let dir_files = util::list_files(&path)?;
            for file in dir_files.iter() {
                index.remove(file);
            }
            if !cached {
                fs::remove_dir_all(&path)?;
            }
        } else {
            index.remove(&path);
            if !cached {
                fs::remove_file(&path)?;
            }
        }
        println!("removed [{}]", file.bright_green());
    }
    Ok(())
}
