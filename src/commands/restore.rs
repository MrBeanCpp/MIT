use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use crate::{
    models::*,
    utils::{util, Store},
};

/// 统计[工作区]中相对于target_blobs已删除的文件（根据filters进行过滤）
fn get_worktree_deleted_files_in_filters(
    filters: &Vec<PathBuf>,
    target_blobs: &HashMap<PathBuf, Hash>,
) -> HashSet<PathBuf> {
    target_blobs //统计所有目录中(包括None & '.')，删除的文件
        .iter()
        .filter(|(path, _)| {
            assert!(path.is_absolute()); //
            !path.exists() && util::include_in_paths(path, filters)
        })
        .map(|(path, _)| path.clone())
        .collect() //HashSet自动去重
}

/// 统计[暂存区index]中相对于target_blobs已删除的文件（根据filters进行过滤）
fn get_index_deleted_files_in_filters(
    index: &Index,
    filters: &Vec<PathBuf>,
    target_blobs: &HashMap<PathBuf, Hash>,
) -> HashSet<PathBuf> {
    target_blobs //统计index中相对target已删除的文件，且包含在指定dir内
        .iter()
        .filter(|(path, _)| {
            assert!(path.is_absolute());
            !index.contains(path) && util::include_in_paths(path, filters)
        })
        .map(|(path, _)| path.clone())
        .collect() //HashSet自动去重
}

/// 将None转化为workdir
fn preprocess_filters(filters: Option<&Vec<PathBuf>>) -> Vec<PathBuf> {
    if let Some(filter) = filters {
        filter.clone()
    } else {
        vec![util::get_working_dir().unwrap()] //None == all(workdir), '.' == cur_dir
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

    let deleted_files = get_worktree_deleted_files_in_filters(&input_paths, &target_blobs); //统计已删除的文件

    let mut file_paths = util::integrate_paths(&input_paths); //根据用户输入整合存在的文件（绝对路径）
    file_paths.extend(deleted_files); //已删除的文件

    let index = Index::get_instance();

    for path in &file_paths {
        assert!(path.is_absolute()); // 绝对路径
        if !path.exists() {
            //文件不存在于workdir
            if target_blobs.contains_key(path) {
                //文件存在于target_commit (deleted)，需要恢复
                Blob::restore(&target_blobs[path], &path);
            } else {
                //在target_commit和workdir中都不存在(非法路径)， 用户输入
                println!("fatal: pathspec '{}' did not match any files", path.display());
            }
        } else {
            //文件存在，有两种情况：1.修改 2.新文件
            if target_blobs.contains_key(path) {
                //文件已修改(modified)
                let file_hash = util::calc_file_hash(&path); //TODO tree没有存修改时间，所以这里只能用hash判断
                if file_hash != target_blobs[path] {
                    Blob::restore(&target_blobs[path], &path);
                }
            } else {
                //新文件，也分两种情况：1.已跟踪，需要删除 2.未跟踪，保留
                if index.tracked(path) {
                    //文件已跟踪
                    fs::remove_file(&path).unwrap();
                    util::clear_empty_dir(&path); // 级联删除 清理空目录
                }
            }
        }
    }
}

/** 根据filter restore staged */
pub fn restore_index(filter: Option<&Vec<PathBuf>>, target_blobs: &Vec<(PathBuf, Hash)>) {
    let input_paths = preprocess_filters(filter); //预处理filter 将None转化为workdir
    let target_blobs = preprocess_blobs(target_blobs); //预处理target_blobs 转化为绝对路径HashMap

    let index = Index::get_instance();
    let deleted_files_index = get_index_deleted_files_in_filters(&index, &input_paths, &target_blobs); //统计已删除的文件

    //1.获取index中包含于input_path的文件（使用paths进行过滤）
    let mut file_paths: HashSet<PathBuf> = util::filter_to_fit_paths(&index.get_tracked_files(), &input_paths);

    // 2.补充index中已删除的文件（相较于target_blobs）
    file_paths.extend(deleted_files_index); //已删除的文件

    for path in &file_paths {
        assert!(path.is_absolute()); // 绝对路径
        if !index.contains(path) {
            //文件不存在于index
            if target_blobs.contains_key(path) {
                //文件存在于target_commit (deleted)，需要恢复
                index.add(path.clone(), FileMetaData { hash: target_blobs[path].clone(), ..Default::default() });
            } else {
                //在target_commit和index中都不存在(非法路径)
                println!("fatal: pathspec '{}' did not match any files", path.display());
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
对于暂存区中被删除的文件，同样会恢复<br>
注意：不会删除空文件夹
 */
pub fn restore(paths: Vec<String>, source: Option<String>, worktree: bool, staged: bool) {
    let paths = paths.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
    let target_commit: Hash = {
        match source {
            None => {
                /*If `--source` not specified, the contents are restored from `HEAD` if `--staged` is given,
                otherwise from the [index].*/
                if staged {
                    head::current_head_commit() // `HEAD`
                } else {
                    Hash::default() //index
                }
            }
            Some(ref src) => {
                if src == "HEAD" {
                    //Default Source
                    head::current_head_commit() // "" if not exist
                } else if head::list_local_branches().contains(&src) {
                    // Branch Name, e.g. master
                    head::get_branch_head(&src) // "" if not exist
                } else {
                    // [Commit Hash, e.g. a1b2c3d4] || [Wrong Branch Name]
                    let store = Store::new();
                    let commit = store.search(&src);
                    if commit.is_none() || !util::is_typeof_commit(commit.clone().unwrap()) {
                        println!("fatal: 非法的 commit hash: '{}'", src);
                        return;
                    }
                    commit.unwrap()
                }
            }
        }
    };

    let target_blobs = {
        /*If `--source` not specified, the contents are restored from `HEAD` if `--staged` is given,
        otherwise from the [index].*/
        if source.is_none() && !staged {
            // 没有指定source，且没有指定--staged，从[index]中恢复到worktree //只有这种情况是从[index]恢复
            let entries = Index::get_instance().get_tracked_entries();
            entries.into_iter().map(|(p, meta)| (p, meta.hash)).collect()
        } else {
            //从[target_commit]中恢复
            if target_commit.is_empty() {
                //target_commit不存在 无法从目标恢复
                if source.is_some() {
                    // 如果指定了source，说明source解析失败，报错
                    println!("fatal: could not resolve {}", source.unwrap());
                    return;
                }
                Vec::new() //否则使用[空]来恢复 代表default status
            } else {
                //target_commit存在，最正常的情况，谢天谢地
                let tree = Commit::load(&target_commit).get_tree();
                tree.get_recursive_blobs() // 相对路径
            }
        }
    };
    // 分别处理worktree和staged
    if worktree {
        restore_worktree(Some(&paths), &target_blobs);
    }
    if staged {
        restore_index(Some(&paths), &target_blobs);
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    //TODO 写测试！
    use crate::{commands as cmd, commands::status, models::Index, utils::test};
    use std::path::PathBuf;

    #[test]
    fn test_restore_stage() {
        test::setup_with_empty_workdir();
        let path = PathBuf::from("a.txt");
        test::ensure_no_file(&path);
        cmd::add(vec![], true, false); //add -A
        cmd::restore(vec![".".to_string()], Some("HEAD".to_string()), false, true);
        let index = Index::get_instance();
        assert!(index.get_tracked_files().is_empty());
    }

    #[test]
    fn test_restore_worktree() {
        test::setup_with_empty_workdir();
        let files = vec!["a.txt", "b.txt", "c.txt", "test/in.txt"];
        test::ensure_files(&files);

        cmd::add(vec![], true, false);
        assert_eq!(status::changes_to_be_committed().new.iter().count(), 4);

        cmd::restore(vec!["c.txt".to_string()], None, false, true); //restore c.txt --staged
        assert_eq!(status::changes_to_be_committed().new.iter().count(), 3);
        assert_eq!(status::changes_to_be_staged().new.iter().count(), 1);

        fs::remove_file("a.txt").unwrap(); //删除a.txt
        fs::remove_dir_all("test").unwrap(); //删除test文件夹
        assert_eq!(status::changes_to_be_staged().deleted.iter().count(), 2);

        cmd::restore(vec![".".to_string()], None, true, false); //restore . //from index
        assert_eq!(status::changes_to_be_committed().new.iter().count(), 3);
        assert_eq!(status::changes_to_be_staged().new.iter().count(), 1);
        assert_eq!(status::changes_to_be_staged().deleted.iter().count(), 0);
        assert!(test::is_file_exist("a.txt"));
        assert!(test::is_file_exist("test/in.txt"));
    }
}
