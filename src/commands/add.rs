use std::env;
use std::path::{Path, PathBuf};
use colored::Colorize;
use crate::models::index::Index;
use crate::utils::util::{check_repo_exist, get_working_dir, list_files};

pub fn add(files: Vec<String>, all: bool, mut update: bool) {
    check_repo_exist();
    let index = Index::new();
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
    }

    for file in files {
        add_a_file(file.as_path(), &index);
    }
}

fn add_a_file(file: &Path, index: &Index) {
    println!("add a file: {:?}", file);
}