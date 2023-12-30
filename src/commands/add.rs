use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::commands::status;
use crate::models::index::FileMetaData;
use crate::models::*;
use crate::utils::util;

/// add是对index的操作，不会对工作区产生影响
pub fn add(raw_paths: Vec<String>, all: bool, mut update: bool) {
    util::check_repo_exist();

    let mut paths: Vec<PathBuf> = raw_paths.into_iter().map(PathBuf::from).collect();
    if all || update {
        println!("{}", "--all || --update 对工作区所有文件进行操作".bright_green());
        paths.push(util::get_working_dir().unwrap());
    }
    if all {
        update = false; // all 优先级最高
    }

    //待暂存的更改： index vs worktree
    let changes = status::changes_to_be_staged().filter_relative(&paths); //对paths过滤
    let mut files = changes.modified;
    //统合所有更改到files再一起处理，其实也可以直接根据changes的分类进行处理 主要是为了错误处理 而且思想上更?简单?
    files.extend(changes.deleted);
    if !update {
        files.extend(changes.new);
    } else {
        println!("{}", "--update 只对已跟踪文件进行操作 不包含new".bright_green());
    }

    let index = Index::get_instance();
    for file in &files {
        add_a_file(file, index);
    }
    index.save();
}

fn add_a_file(file: &Path, index: &mut Index) {
    let workdir = util::get_working_dir().unwrap();
    if !util::is_sub_path(file, &workdir) {
        //文件不在工作区内
        println!("fatal: '{}' is outside workdir at '{}'", file.display(), workdir.display());
        return;
    }
    if util::is_inside_repo(file) {
        //文件在.mit内
        println!("fatal: '{}' is inside '{}' repo", file.display(), util::ROOT_DIR);
        return;
    }

    let rel_path = util::get_relative_path(file);
    if !file.exists() {
        //文件被删除
        index.remove(file);
        println!("removed: {}", rel_path.display());
    } else {
        //文件存在
        if !index.contains(file) {
            //文件未被跟踪
            let blob = Blob::new(util::read_workfile(file));
            index.add(file.to_path_buf(), FileMetaData::new(&blob, file));
            println!("add(stage): {}", rel_path.display());
        } else {
            //文件已被跟踪，可能被修改
            if index.is_modified(file) {
                //文件被修改，但不一定内容更改
                let blob = Blob::new(util::read_workfile(file)); //到这一步才创建blob是为了优化
                if !index.verify_hash(file, &blob.get_hash()) {
                    //比较hash 确认内容更改
                    index.update(file.to_path_buf(), FileMetaData::new(&blob, file));
                    println!("add(modified): {}", rel_path.display());
                }
            }
        }
    }
}
