use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use crate::{
    head,
    models::{
        commit::Commit,
        index::{FileMetaData, Index},
        object::Hash,
    },
    store::Store,
    utils::{util, util::get_working_dir},
};

/// 统计[工作区]中的dirs文件夹中，相对于target_blobs已删除的文件
fn get_worktree_deleted_files_in_dirs(
    dirs: &HashSet<PathBuf>,
    target_blobs: &HashMap<PathBuf, Hash>,
) -> HashSet<PathBuf> {
    target_blobs //统计所有目录中(包括None & '.')，删除的文件
        .iter()
        .filter(|(path, _)| {
            assert!(path.is_absolute()); //
            if !path.exists() {
                for dir in dirs {
                    if util::is_parent_dir(path, dir) {
                        //需要包含在指定dir内
                        return true;
                    }
                }
            }
            false
        })
        .map(|(path, _)| path.clone())
        .collect() //HashSet自动去重
}

/// 统计[暂存区index]中相对于target_blobs已删除的文件，且包含在指定dirs内
fn get_index_deleted_files_in_dirs(
    index: &Index,
    dirs: &HashSet<PathBuf>,
    target_blobs: &HashMap<PathBuf, Hash>,
) -> HashSet<PathBuf> {
    target_blobs //统计index中相对target已删除的文件，且包含在指定dir内
        .iter()
        .filter(|(path, _)| {
            assert!(path.is_absolute()); //
            if !index.contains(path) {
                //index中不存在
                for dir in dirs {
                    if util::is_parent_dir(path, dir) {
                        //需要包含在指定dir内
                        return true;
                    }
                }
            }
            false
        })
        .map(|(path, _)| path.clone())
        .collect() //HashSet自动去重
}

/// 将None转化为workdir
fn preprocess_filters(filters: Option<&Vec<PathBuf>>) -> Vec<PathBuf> {
    if let Some(filter) = filters {
        filter.clone()
    } else {
        vec![get_working_dir().unwrap()] //None == all(workdir), '.' == cur_dir
    }
}

/// 转化为绝对路径（to workdir）的HashMap
fn preprocess_blobs(blobs: &Vec<(PathBuf, Hash)>) -> HashMap<PathBuf, Hash> {
    blobs // 转为绝对路径 //TODO tree改变路径表示方式后，这里需要修改
        .iter()
        .map(|(path, hash)| (util::to_workdir_absolute_path(path), hash.clone()))
        .collect() //to HashMap
}

/** 根据filter restore workdir */
pub fn restore_worktree(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    let input_paths = preprocess_filters(filter); //预处理filter 将None转化为workdir
    let target_blobs = preprocess_blobs(target_blobs); //预处理target_blobs 转化为绝对路径HashMap

    let dirs = util::filter_dirs(&input_paths); //统计所有目录
    let deleted_files = get_worktree_deleted_files_in_dirs(&dirs, &target_blobs); //统计所有目录中已删除的文件

    let mut file_paths = util::integrate_paths(&input_paths); //整合存在的文件（绝对路径）
    file_paths.extend(deleted_files); //已删除的文件

    let index = Index::new();
    let store = Store::new();

    for path in &file_paths {
        assert!(path.is_absolute() && !path.is_dir()); // 绝对路径且不是目录
        if !path.exists() {
            //文件不存在于workdir
            if target_blobs.contains_key(path) {
                //文件存在于target_commit (deleted)，需要恢复
                store.restore_to_file(&target_blobs[path], &path);
            } else {
                //在target_commit和workdir中都不存在(非法路径)
                // println!("fatal: pathspec '{}' did not match any files", path.display());
                // TODO 如果是用户输入的路径，才应该报错，integrate_paths产生的不应该报错
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
/** 根据filter restore staged */
pub fn restore_index(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    let input_paths = preprocess_filters(filter); //预处理filter 将None转化为workdir
    let target_blobs = preprocess_blobs(target_blobs); //预处理target_blobs 转化为绝对路径HashMap

    let mut index = Index::new();

    let dirs = util::filter_dirs(&input_paths); //统计所有目录
    let deleted_files_index = get_index_deleted_files_in_dirs(&index, &dirs, &target_blobs); //统计所有目录中已删除的文件

    // 1. 获取输入中的[文件路径]
    let mut file_paths: HashSet<PathBuf> = util::filter_files(&input_paths)
        .iter()
        .map(|path| util::get_absolute_path(path))
        .collect();
    // 2.获取index中包含于dirs的文件（使用dirs进行筛选）
    for index_file in index.get_tracked_files() {
        for dir in &dirs {
            if util::is_parent_dir(&index_file, dir) {
                //需要包含在指定dir内
                file_paths.insert(index_file.clone());
            }
        }
    }
    // 3.补充index中已删除的文件（相较于target_blobs）
    file_paths.extend(deleted_files_index); //已删除的文件

    for path in &file_paths {
        assert!(path.is_absolute() && !path.is_dir()); // 绝对路径且不是目录
        if !index.contains(path) {
            //文件不存在于index
            if target_blobs.contains_key(path) {
                //文件存在于target_commit (deleted)，需要恢复
                index.add(path.clone(), FileMetaData { hash: target_blobs[path].clone(), ..Default::default() });
            } else {
                //在target_commit和index中都不存在(非法路径)
                // println!("fatal: pathspec '{}' did not match any files", path.display());
                // TODO 如果是用户输入的路径，才应该报错，integrate_paths产生的不应该报错
            }
        } else {
            //文件存在于index，有两种情况：1.修改 2.新文件
            if target_blobs.contains_key(path) {
                if !index.verify_hash(path, &target_blobs[path]) {
                    //文件已修改(modified)
                    index.update(path.clone(), FileMetaData { hash: target_blobs[path].clone(), ..Default::default() });
                }
            } else {
                //新文件 需要从index中删除
                index.remove(path);
            }
        }
    }
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
