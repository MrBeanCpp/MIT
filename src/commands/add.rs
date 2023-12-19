use std::env;
use std::path::{Path, PathBuf};
use colored::Colorize;
use sha1::Digest;
use crate::models::blob::Blob;
use crate::models::index::{FileMetaData, Index};
use crate::utils::util::{check_repo_exist, get_relative_path, get_working_dir, list_files};

pub fn add(files: Vec<String>, all: bool, mut update: bool) {
    check_repo_exist();
    let mut index = Index::new();
    let mut dot = files.contains(&".".to_string());

    let mut files: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();

    if dot || all || update{
        if all { // all 优先级最高
            dot = false;
            update = false;
        } else if update {
            dot = false;
        }

        let path = if all || update {
            println!("{}", "--all || --update 运行于工作区目录".bright_green());
            get_working_dir().unwrap()
        } else {
            println!("{}", "'.'代表了当前目录".bright_green());
            env::current_dir().unwrap()
        };
        println!("Working on [{}]\n", path.to_str().unwrap().bright_blue());
        files = list_files(&*path).unwrap();
        if update {
            files.retain(|file|{
                index.contains(file)
            });
        }

        files.extend(index.get_deleted_files()); //包含已删除的文件
    }

    for file in &files {
        add_a_file(file, &mut index);
    }

    index.save();
}

fn add_a_file(file: &Path, index: &mut Index) {
    println!("add a file: {}", get_relative_path(file, get_working_dir().unwrap()).display());
    if !file.exists() { //文件被删除
        index.remove(file);
    } else {
        let blob = Blob::new(file);
        let file_data = FileMetaData::new(&blob, file);
        if !index.contains(file) { //文件未被跟踪
            index.add(file.to_path_buf(), file_data);
        } else { //文件已被跟踪，可能被修改
            index.update(file.to_path_buf(), file_data);
        }
    }
}