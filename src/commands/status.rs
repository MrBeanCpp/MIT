use std::path::PathBuf;

use crate::utils::util::to_workdir_absolute_path;
use crate::{
    head,
    models::{blob, commit, index},
    utils::util,
};

/** 获取需要commit的更改(staged) */
#[derive(Debug, Default)]
pub struct Changes {
    pub new: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

pub fn changes_to_be_committed() -> Changes {
    let mut change = Changes::default();
    let index = index::Index::new();
    let head_hash = head::current_head_commit();
    let tracked_files = index
        .get_tracked_files()
        .iter()
        .map(|f| util::to_root_relative_path(f))
        .collect::<Vec<PathBuf>>();
    if head_hash == "" {
        // 初始提交
        change.new = tracked_files;
        return change;
    }

    let commit = commit::Commit::load(&head_hash);
    let tree = commit.get_tree();
    let tree_files = tree.get_recursive_blobs(); //相对路径
    let index_files: Vec<PathBuf> = tracked_files;

    for (tree_file, blob_hash) in tree_files.iter() {
        let index_file = index_files.iter().find(|&f| f == tree_file);
        if let Some(index_file) = index_file {
            let index_path = to_workdir_absolute_path(index_file);
            if !index.verify_hash(&index_path, blob_hash) {
                change.modified.push(tree_file.clone());
            }
        } else {
            change.deleted.push(tree_file.clone()); //todo: abs_path?
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

pub fn changes_to_be_staged() {
    unimplemented!()
}

/** 分为两个部分
1. unstaged: 暂存区与工作区比较
2. staged to be committed: 暂存区与HEAD(最后一次Commit::Tree)比较，即上次的暂存区
 */
pub fn status() {
    unimplemented!()
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
