use std::path::PathBuf;

use crate::{
    head,
    models::{commit::Commit, object::Hash},
    store::Store,
};

/** 根据filte restore workdir */
pub fn restore_workdir_into_files(filter: Option<PathBuf>, target_blobs: Vec<(PathBuf, Hash)>) {
    // TODO
    unimplemented!("TODO");
}
/** 根据filte restore staged */
pub fn restore_staged_into_files(filter: Option<PathBuf>, target_blobs: Vec<(PathBuf, Hash)>) {
    // TODO
    unimplemented!("TODO");
}

pub fn restore(path: Vec<String>, source: String, worktree: bool, staged: bool) {
    // TODO
    let target_commit = {
        if source == "HEAD" {
            head::current_head_commit()
        } else if head::list_local_branches().contains(&source) {
            head::get_branch_head(&source)
        } else {
            let store = Store::new();
            let commit = store.search(&source);
            if commit.is_none() {
                println!("fatal: 非法的 commit: '{}'", source);
                return;
            }
            commit.unwrap()
        }
    };
    // TODO 处理筛选path的互相包含的情况

    // 分别处理worktree和staged
    let tree = Commit::load(&target_commit).get_tree();
    let target_blobs = tree.get_recursive_blobs();
    if worktree {
        unimplemented!("TODO")
    }
    if staged {
        unimplemented!("TODO")
    }
    unimplemented!("TODO");
}
