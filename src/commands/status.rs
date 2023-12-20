use std::path::PathBuf;

use crate::{head, models::index, utils::util};

/** 获取需要commit的更改 */
pub struct Changes {
    pub new: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

fn __file_string(path: &PathBuf) -> String {
    util::to_root_relative_path(&path)
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn changes_to_be_committed() -> Changes {
    let mut change = Changes {
        new: vec![],
        modified: vec![],
        deleted: vec![],
    };
    let index = index::Index::new();
    let head_hash = head::current_head_commit();
    if head_hash == "".to_string() {
        // 初始提交
        change.new = index
            .get_tracked_files()
            .iter()
            .map(|f| __file_string(f))
            .collect();
        return change;
    }

    let commit = crate::models::commit::Commit::load(&head_hash);
    let tree = commit.get_tree();
    let mut tree_files = tree.get_recursive_blobs();
    let mut index_files: Vec<PathBuf> = index
        .get_tracked_files()
        .iter()
        .map(|f| util::to_root_relative_path(f))
        .collect();

    for tree_item in tree_files.iter() {
        if index_files.contains(&tree_item.0) {
            // 比较文件内容
            // XXX @mrbeanc 我看到Blob的new被改成调用save了。这里的实现希望比较Blob内容，不然就得读取文件内容。
        } else {
            change.deleted.push(__file_string(&tree_item.0));
        }
    }

    change
}

pub fn status() {
    unimplemented!()
}
