use std::collections::{HashMap, HashSet};
use std::{fs, path::PathBuf};

use crate::utils::util::{get_absolute_path, list_files};
use crate::{
    head,
    models::{commit::Commit, index::Index, object::Hash},
    store::Store,
    utils::{util, util::get_working_dir},
};

/** 根据filter restore workdir */
pub fn restore_worktree(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    let paths: Vec<PathBuf> = if let Some(filter) = filter {
        filter.clone()
    } else {
        vec![get_working_dir().unwrap()] //None == all(workdir), '.' == cur_dir
    };

    let target_blobs = target_blobs // 转为绝对路径 //TODO tree改变路径表示方式后，这里需要修改
        .iter()
        .map(|(path, hash)| (util::to_workdir_absolute_path(path), hash.clone()))
        .collect::<HashMap<PathBuf, Hash>>();

    let dirs: Vec<PathBuf> = paths.iter().filter(|path| path.is_dir()).cloned().collect();
    let del_files = target_blobs //统计所有目录中(包括None & '.')，删除的文件
        .iter()
        .filter(|(path, _)| {
            if !path.exists() {
                for dir in &dirs {
                    if util::is_parent_dir(path, dir) {
                        //需要包含在指定dir内
                        return true;
                    }
                }
            }
            false
        })
        .map(|(path, _)| path.clone())
        .collect::<HashSet<PathBuf>>(); //HashSet自动去重
    let mut paths = util::integrate_paths(&paths); //存在的文件路径
    paths.extend(del_files); //不存在的文件路径

    let index = Index::new();
    let store = Store::new();

    for path in &paths {
        assert!(path.is_absolute() && !path.is_dir()); // 绝对路径且不是目录
        if !path.exists() {
            //文件不存在于workdir
            if target_blobs.contains_key(path) {
                //文件存在于target_commit
                store.restore_to_file(&target_blobs[path], &path);
            } else {
                //在target_commit和workdir中都不存在(非法路径)
                println!("fatal: pathspec '{}' did not match any files", path.display());
            }
        } else {
            //文件存在，有两种情况：1.修改 2.新文件
            if target_blobs.contains_key(path) {
                //文件已修改(modified)
                let file_hash = util::calc_file_hash(&path); //TODO tree没有存修改时间，所以这里只能用hash判断
                if file_hash != target_blobs[path] {
                    store.restore_to_file(&target_blobs[path], &path);
                }
            } else {
                //新文件，也分两种情况：1.已跟踪，需要删除 2.未跟踪，保留
                if index.tracked(path) {
                    //文件已跟踪
                    fs::remove_file(&path).unwrap();
                }
            }
        }
    }
}
/** 根据filte restore staged */
pub fn restore_index(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    // TODO 让@mrbeanc来写吧
    unimplemented!("TODO");
}
/**
对于工作区中的新文件，若已跟踪，则删除；若未跟踪，则保留<br>
对于暂存区中被删除的文件，同样会恢复
 */
pub fn restore(paths: Vec<String>, source: String, worktree: bool, staged: bool) {
    // TODO 尝试合并restore_index和restore_worktree（逻辑上是一致的）
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
