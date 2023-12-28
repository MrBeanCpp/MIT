#![cfg(test)]

pub const TEST_DIR: &str = "mit_test_storage";
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::models::Index;

// 执行测试的储存库
use super::util;
/* tools for test */
fn find_cargo_dir() -> PathBuf {
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR");
    if cargo_path.is_err() {
        // vscode DEBUG test没有CARGO_MANIFEST_DIR宏，手动尝试查找cargo.toml
        let mut path = util::cur_dir();
        loop {
            path.push("Cargo.toml");
            if path.exists() {
                break;
            }
            if !path.pop() {
                panic!("找不到CARGO_MANIFEST_DIR");
            }
        }
        path.pop();
        path
    } else {
        PathBuf::from(cargo_path.unwrap())
    }
}

/// 准备测试环境，切换到测试目录
fn setup_test_env() {
    color_backtrace::install(); // colorize backtrace

    let mut path = find_cargo_dir();
    path.push(TEST_DIR);
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    std::env::set_current_dir(&path).unwrap(); // 将执行目录切换到测试目录
}

pub fn init_mit() {
    let _ = crate::commands::init();
    Index::reload(); // 重置index, 以防止其他测试修改了index单例
}

/// with 初始化的干净的mit
pub fn setup_test_with_clean_mit() {
    setup_test_without_mit();
    init_mit();
}

pub fn setup_test_without_mit() {
    // 将执行目录切换到测试目录，并清除测试目录下的.mit目录
    setup_test_env();
    let mut path = util::cur_dir();
    path.push(util::ROOT_DIR);
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
}

pub fn ensure_test_files<T: AsRef<str>>(paths: &Vec<T>) {
    for path in paths {
        ensure_test_file(path.as_ref().as_ref(), None);
    }
}

pub fn ensure_empty_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let entries = fs::read_dir(path.as_ref())?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?; // 如果是目录，则递归删除
        } else {
            fs::remove_file(&path)?; // 如果是文件，则直接删除
        }
    }
    Ok(())
}

pub fn setup_test_with_empty_workdir() {
    let test_dir = find_cargo_dir().join(TEST_DIR);
    ensure_empty_dir(&test_dir).unwrap();
    setup_test_with_clean_mit();
}

pub fn ensure_test_file(path: &Path, content: Option<&str>) {
    // 以测试目录为根目录，创建文件
    fs::create_dir_all(path.parent().unwrap()).unwrap(); // ensure父目录
    let mut file = fs::File::create(util::get_working_dir().unwrap().join(path))
        .expect(format!("无法创建文件：{:?}", path).as_str());
    if let Some(content) = content {
        file.write(content.as_bytes()).unwrap();
    } else {
        // 写入文件名
        file.write(path.file_name().unwrap().to_str().unwrap().as_bytes()).unwrap();
    }
}

pub fn ensure_no_file(path: &Path) {
    // 以测试目录为根目录，删除文件
    if path.exists() {
        fs::remove_file(util::get_working_dir().unwrap().join(path)).unwrap();
    }
}
