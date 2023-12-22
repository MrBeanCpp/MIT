use std::path::PathBuf;

use colored::Colorize;

use crate::{
    head::{self},
    models::{commit::Commit, object::Hash},
    store::Store,
    utils::util,
};

use super::{
    branch,
    restore::{restore_staged_into_files, restore_workdir_into_files},
    status::{changes_to_be_committed, changes_to_be_staged},
};

enum SwitchErr {
    NoClean,
    InvalidBranch,
    InvalidObject,
}

/** 将工作区域的文件更改为commit_hash的版本，可以指定filter未特定文件或路径 */
fn switch_to_commit(commit_hash: Hash) {
    let commit = Commit::load(&commit_hash);
    let tree = commit.get_tree();
    let target_files = tree.get_recursive_blobs(); // 相对路径

    // 借用逻辑类似的restore_workdir_into_files
    restore_workdir_into_files(None, target_files.clone());
    // 同时restore index
    restore_staged_into_files(None, target_files);
}

fn switch_to(branch: String, detach: bool) -> Result<(), SwitchErr> {
    // 检查更改
    if !changes_to_be_staged().is_empty() {
        println!("fatal: 你有未暂存的更改，切换分支会导致更改丢失");
        return Err(SwitchErr::NoClean);
    } else if !changes_to_be_committed().is_empty() {
        println!("fatal: 你有未提交的更改，无法切换分支");
        return Err(SwitchErr::NoClean);
    }

    let store = Store::new();
    if head::list_local_branches().contains(&branch) {
        // 切到分支
        let branch_commit = head::get_branch_head(&branch);
        switch_to_commit(branch_commit.clone());
        head::change_head_to_branch(&branch); // 更改head
        println!("切换到分支： '{}'", branch.green())
    } else if detach {
        let commit = store.search(&branch);
        if commit.is_none() || util::check_object_type(commit.clone().unwrap()) != util::ObjectType::Commit {
            println!("fatal: 非法的 commit: '{}'", branch);
            return Err(SwitchErr::InvalidObject);
        }

        // 切到commit
        let commit = commit.unwrap();
        switch_to_commit(commit.clone());
        head::change_head_to_commit(&commit); // 更改head
        println!("切换到 detach commit： '{}'", commit.yellow())
    } else {
        println!("fatal: 不存在分支 '{}'", branch);
        return Err(SwitchErr::InvalidBranch);
    }

    Ok(())
}

pub fn switch(target_branch: Option<String>, create: Option<String>, detach: bool) {
    match create {
        Some(new_branch) => {
            // 以target_branch为基础创建新分支create
            println!("create new branch: {:?}", new_branch);
            branch::branch(Some(new_branch.clone()), target_branch.clone(), false, None, false);
            let _ = switch_to(new_branch, true);
        }
        None => {
            println!("switch to branch: {:?}", target_branch.as_ref().unwrap());
            let _ = switch_to(target_branch.unwrap(), detach);
        }
    }
}
