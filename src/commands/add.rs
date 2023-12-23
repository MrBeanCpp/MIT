use std::{
    env,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    models::{
        blob::Blob,
        index::{FileMetaData, Index},
    },
    utils::util::{
        check_repo_exist, get_working_dir, is_inside_repo, is_inside_workdir, list_files, to_workdir_relative_path,
        ROOT_DIR,
    },
};

pub fn add(files: Vec<String>, all: bool, mut update: bool) {
    check_repo_exist();
    let mut index = Index::new();
    let mut dot = files.contains(&".".to_string());

    let mut files: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();

    if dot || all || update {
        //TODO files中可能包含文件夹，需要统计文件夹中被删除的文件
        if all {
            // all 优先级最高
            dot = false;
            update = false;
        } else if update {
            // update 优先级次之
            dot = false;
        }

        let dir = if all || update {
            println!("{}", "--all || --update 运行于工作区目录".bright_green());
            get_working_dir().unwrap()
        } else if dot {
            println!("{}", "'.'代表了当前目录".bright_green());
            env::current_dir().unwrap()
        } else {
            panic!();
        };

        println!("Working on [{}]\n", dir.to_str().unwrap().bright_blue());
        files = list_files(&dir).unwrap();
        if update {
            files.retain(|file| index.contains(file));
        }

        files.extend(index.get_deleted_files(&dir)); //包含已删除的文件
    }

    for file in &files {
        add_a_file(file, &mut index);
    }
}

fn add_a_file(file: &Path, index: &mut Index) {
    if !is_inside_workdir(file) && file.exists() {
        //文件不在工作区内
        println!("fatal: '{}' is outside repository at '{}'", file.display(), get_working_dir().unwrap().display());
        return;
    }
    if is_inside_repo(file) {
        //文件在.mit内
        println!("fatal: '{}' is inside '{}' repo", file.display(), ROOT_DIR);
        return;
    }

    let relative_path = to_workdir_relative_path(file);
    if !file.exists() {
        //文件被删除
        index.remove(file);
        println!("removed: {}", relative_path.display());
    } else {
        //文件存在
        if !index.contains(file) {
            //文件未被跟踪
            let blob = Blob::new(file);
            index.add(file.to_path_buf(), FileMetaData::new(&blob, file));
            println!("add(stage): {}", relative_path.display());
        } else {
            //文件已被跟踪，可能被修改
            if index.is_modified(file) {
                //文件被修改，但不一定内容更改
                let blob = Blob::new(file); //到这一步才创建blob是为了优化
                if !index.verify_hash(file, &blob.get_hash()) {
                    //比较hash 确认内容更改
                    index.update(file.to_path_buf(), FileMetaData::new(&blob, file));
                    println!("add(modified): {}", relative_path.display());
                }
            }
        }
    }
}
