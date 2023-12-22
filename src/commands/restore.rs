use std::fs;
use std::path::PathBuf;

use crate::models::index::Index;
use crate::utils::util;
use crate::utils::util::get_working_dir;
use crate::{
    head,
    models::{commit::Commit, object::Hash},
    store::Store,
};

/** 根据filter restore workdir */
pub fn restore_worktree(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    let paths: Vec<PathBuf> = if let Some(filter) = filter {
        filter.clone()
    } else {
        vec![get_working_dir().unwrap()] //None == all(workdir), '.' == cur_dir
    };
    let paths = util::integrate_paths(&paths); // file paths

    let target_blobs = target_blobs // 转为相对路径(cur_dir) 与filter统一 //TODO tree改变路径表示方式后，这里需要修改
        .iter()
        .map(|(path, hash)| (util::to_workdir_relative_path(path), hash.clone()))
        .collect::<Vec<(PathBuf, Hash)>>();

    let index = Index::new();
    let store = Store::new();
    for (path, hash) in &target_blobs {
        if path.exists() {
            let file_hash = util::calc_file_hash(&path); //TODO tree没有存修改时间，所以这里只能用hash判断
            if file_hash == *hash {
                continue; //文件未修改 不需要还原
            }
        }
        //文件不存在或已修改
        store.restore_to_file(hash, &path);
    }

    //处理工作区的新文件
    for path in paths {
        if target_blobs.iter().any(|(target_path, _)| target_path == &path) {
            //TODO 最好返回HashMap 方便优化
            continue; //已处理
        }
        //未找到，则对于target_commit来说是新文件；若已跟踪，则删除；若未跟踪，则保留
        if index.tracked(&path) {
            fs::remove_file(&path).unwrap();
        }
    }
}
/** 根据filte restore staged */
pub fn restore_index(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    // TODO
    unimplemented!("TODO");
}
/**
对于工作区中的新文件，若已跟踪，则删除；若未跟踪，则保留
 */
pub fn restore(paths: Vec<String>, source: String, worktree: bool, staged: bool) {
    // TODO
    let paths = paths.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
    let target_commit: Hash = {
        if source == "HEAD" {
            //Default
            head::current_head_commit()
        } else if head::list_local_branches().contains(&source) {
            // Branch Name, e.g. master
            head::get_branch_head(&source)
        } else {
            // Commit Hash, e.g. a1b2c3d4
            let store = Store::new();
            let commit = store.search(&source);
            if commit.is_none() || !util::is_typeof_commit(commit.clone().unwrap()) {
                println!("fatal: 非法的 commit hash: '{}'", source);
                return;
            }
            commit.unwrap()
        }
    };

    // 分别处理worktree和staged
    let tree = Commit::load(&target_commit).get_tree();
    let target_blobs = tree.get_recursive_blobs(); // 相对路径
    if worktree {
        restore_worktree(Some(&paths), &target_blobs);
    }
    if staged {
        restore_index(Some(&paths), &target_blobs);
    }
}
