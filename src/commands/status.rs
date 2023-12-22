use crate::{
    head,
    head::Head,
    models::{commit::Commit, index::Index},
    utils::{util, util::check_repo_exist},
};
use colored::Colorize;
use std::path::PathBuf;

/** 获取需要commit的更改(staged)
   注：相对路径
*/
#[derive(Debug, Default)]
pub struct Changes {
    pub new: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.new.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }
}

pub fn changes_to_be_committed() -> Changes {
    let mut change = Changes::default();
    let index = Index::new();
    let head_hash = head::current_head_commit();
    let tracked_files = index
        .get_tracked_files()
        .iter()
        .map(|f| util::to_workdir_relative_path(f))
        .collect::<Vec<PathBuf>>();
    if head_hash == "" {
        // 初始提交
        change.new = tracked_files;
        return change;
    }

    let commit = Commit::load(&head_hash);
    let tree = commit.get_tree();
    let tree_files = tree.get_recursive_blobs(); //相对路径
    let index_files: Vec<PathBuf> = tracked_files;

    for (tree_file, blob_hash) in tree_files.iter() {
        let index_file = index_files.iter().find(|&f| f == tree_file);
        if let Some(index_file) = index_file {
            let index_path = util::to_workdir_absolute_path(index_file);
            if !index.verify_hash(&index_path, blob_hash) {
                change.modified.push(tree_file.clone());
            }
        } else {
            change.deleted.push(tree_file.clone());
        }
    }
    for index_file in index_files.iter() {
        let tree_item = tree_files.iter().find(|f| f.0 == *index_file);
        if tree_item.is_none() {
            change.new.push(index_file.clone());
        }
    }
    change
}

pub fn changes_to_be_staged() -> Changes {
    let mut change = Changes::default();
    let index = Index::new();
    for file in index.get_tracked_files() {
        //TODO 考虑当前目录
        if !file.exists() {
            change.deleted.push(util::to_workdir_relative_path(&file));
        } else if index.is_modified(&file) {
            // 若文件元数据被修改，才需要比较暂存区与文件的hash来判别内容修改
            if !index.verify_hash(&file, &util::calc_file_hash(&file)) {
                change.modified.push(util::to_workdir_relative_path(&file));
            }
        }
    }
    let files = util::list_workdir_files(); //TODO 考虑当前目录
    for file in files {
        if !index.tracked(&file) {
            //文件未被跟踪
            change.new.push(util::to_workdir_relative_path(&file));
        }
    }

    change
}

/** 分为两个部分
1. unstaged: 暂存区与工作区比较
2. staged to be committed: 暂存区与HEAD(最后一次Commit::Tree)比较，即上次的暂存区
 */
pub fn status() {
    check_repo_exist();
    //TODO: 输出文件与当前所在目录有关 输出时过滤
    match head::current_head() {
        Head::Detached(commit) => {
            println!("HEAD detached at {}", commit[0..7].to_string());
        }
        Head::Branch(branch) => {
            println!("On branch {}", branch);
        }
    }

    let staged = changes_to_be_committed();
    let unstaged = changes_to_be_staged();
    if staged.is_empty() && unstaged.is_empty() {
        println!("nothing to commit, working tree clean");
        return;
    }

    if !staged.is_empty() {
        println!("Changes to be committed:");
        println!("  use \"mit restore --staged <file>...\" to unstage");
        staged.deleted.iter().for_each(|f| {
            let str = format!("\tdeleted: {}", f.display());
            println!("{}", str.bright_green());
        });
        staged.modified.iter().for_each(|f| {
            let str = format!("\tmodified: {}", f.display());
            println!("{}", str.bright_green());
        });
        staged.new.iter().for_each(|f| {
            let str = format!("\tnew file: {}", f.display());
            println!("{}", str.bright_green());
        });
    }

    if !unstaged.deleted.is_empty() || !unstaged.modified.is_empty() {
        println!("Changes not staged for commit:");
        println!("  use \"mit add <file>...\" to update what will be committed");
        unstaged.deleted.iter().for_each(|f| {
            let str = format!("\tdeleted: {}", f.display());
            println!("{}", str.bright_red());
        });
        unstaged.modified.iter().for_each(|f| {
            let str = format!("\tmodified: {}", f.display());
            println!("{}", str.bright_red());
        });
    }
    if !unstaged.new.is_empty() {
        println!("Untracked files:");
        println!("  use \"mit add <file>...\" to include in what will be committed");
        unstaged.new.iter().for_each(|f| {
            let str = format!("\t{}", f.display());
            println!("{}", str.bright_red());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::{self, commit},
        utils::util,
    };
    use std::path::Path;

    #[test]
    fn test_changes_to_be_committed() {
        util::setup_test_with_clean_mit();
        let test_file = "a.txt";
        util::ensure_test_file(Path::new(test_file), None);

        commit::commit("test commit".to_string(), true);
        commands::add::add(vec![test_file.to_string()], false, false);
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 1);
        assert_eq!(change.modified.len(), 0);
        assert_eq!(change.deleted.len(), 0);

        println!("{:?}", change);

        commit::commit("test commit".to_string(), true);
        util::ensure_test_file(Path::new(test_file), Some("new content"));
        commands::add::add(vec![test_file.to_string()], false, false);
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 0);
        assert_eq!(change.modified.len(), 1);
        assert_eq!(change.deleted.len(), 0);

        println!("{:?}", change);

        commit::commit("test commit".to_string(), true);
        let _ = commands::remove::remove(vec![test_file.to_string()], false, false);
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 0);
        assert_eq!(change.modified.len(), 0);
        assert_eq!(change.deleted.len(), 1);

        println!("{:?}", change);
    }
}
