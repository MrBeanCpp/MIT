use std::path::PathBuf;

use crate::{
    head,
    models::{blob, index},
    utils::util,
};

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
    let tree_files = tree.get_recursive_blobs();
    let index_files: Vec<PathBuf> = index
        .get_tracked_files()
        .iter()
        .map(|f| util::to_root_relative_path(f))
        .collect();

    for tree_item in tree_files.iter() {
        let index_file = index_files.iter().find(|f| **f == tree_item.0);
        if index_file.is_none() {
            change.deleted.push(__file_string(&tree_item.0));
        } else {
            let index_blob = blob::Blob::new(
                util::get_working_dir()
                    .unwrap()
                    .join(index_file.unwrap())
                    .as_path(),
            );
            // XXX @mrbeanc 我看到Blob的new被改成调用save了。这里的实现希望比较Blob内容，不然就得读取文件内容。
            if index_blob.get_hash() != tree_item.1.get_hash() {
                change.modified.push(__file_string(&tree_item.0));
            }
        }
    }
    for index_file in index_files.iter() {
        let tree_item = tree_files.iter().find(|f| f.0 == **index_file);
        if tree_item.is_none() {
            change.new.push(__file_string(&index_file));
        }
    }
    change
}

pub fn status() {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::commit, utils::util};
    use std::{fs, path::Path};

    #[test]
    fn test_changes_to_be_committed() {
        util::setup_test_with_clean_mit();
        let test_file = "a.txt";
        util::ensure_test_file(Path::new(test_file), None);

        commit::commit("test commit".to_string(), true);
        let mut index = index::Index::new();
        index.add(
            PathBuf::from(test_file),
            index::FileMetaData::new(&blob::Blob::new(Path::new(test_file)), Path::new(test_file)),
        );
        index.save();
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 1);
        assert_eq!(change.modified.len(), 0);
        assert_eq!(change.deleted.len(), 0);

        commit::commit("test commit".to_string(), true);
        util::ensure_test_file(Path::new(test_file), Some("new content"));
        index.add(
            PathBuf::from(test_file),
            index::FileMetaData::new(&blob::Blob::new(Path::new(test_file)), Path::new(test_file)),
        );
        index.save();
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 0);
        assert_eq!(change.modified.len(), 1);
        assert_eq!(change.deleted.len(), 0);

        commit::commit("test commit".to_string(), true);
        index.remove(
            util::get_working_dir()
                .unwrap()
                .join(Path::new(test_file))
                .as_path(),
        );
        index.save();
        let change = changes_to_be_committed();
        assert_eq!(change.new.len(), 0);
        assert_eq!(change.modified.len(), 0);
        assert_eq!(change.deleted.len(), 1);
    }
}
