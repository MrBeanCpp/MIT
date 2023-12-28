use colored::Colorize;

use crate::{
    models::{Commit, Hash},
    utils::{head, store, util},
};

use super::{
    branch,
    restore::{restore_index, restore_worktree},
    status,
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
    restore_worktree(None, &target_files);
    // 同时restore index
    restore_index(None, &target_files);
}

fn switch_to(branch: String, detach: bool) -> Result<(), SwitchErr> {
    // 检查更改
    if !status::changes_to_be_staged().is_empty() {
        status::status();
        println!("fatal: 你有未暂存的更改，切换分支会导致更改丢失");
        return Err(SwitchErr::NoClean);
    } else if !status::changes_to_be_committed().is_empty() {
        status::status();
        println!("fatal: 你有未提交的更改，无法切换分支");
        return Err(SwitchErr::NoClean);
    }

    let store = store::Store::new();
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        commands::{self as cmd},
        utils::test_util,
    };
    use std::path::PathBuf;
    #[test]
    fn test_switch() {
        test_util::setup_test_with_empty_workdir();

        cmd::commit("init".to_string(), true);
        let test_branch_1 = "test_branch_1".to_string();
        cmd::branch(Some(test_branch_1.clone()), None, false, None, false);

        /* test 1: NoClean */
        let test_file_1 = PathBuf::from("test_file_1");
        test_util::ensure_test_file(&test_file_1, None);
        let result = switch_to(test_branch_1.clone(), false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SwitchErr::NoClean));

        cmd::add(vec![], true, false); // add all
        cmd::commit("add file 1".to_string(), true);
        let test_branch_2 = "test_branch_2".to_string();
        cmd::branch(Some(test_branch_2.clone()), None, false, None, false); // branch2: test_file_1 exists

        /* test 2: InvalidBranch */
        let result = switch_to("invalid_branch".to_string(), false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SwitchErr::InvalidBranch));

        /* test 3: InvalidObject*/
        let result = switch_to("invalid_commit".to_string(), true);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SwitchErr::InvalidObject));

        let tees_file_2 = PathBuf::from("test_file_2");
        test_util::ensure_test_file(&tees_file_2, None);
        cmd::add(vec![], true, false); // add all
        cmd::commit("add file 2".to_string(), false);
        let history_commit = head::current_head_commit(); // commit: test_file_1 exists, test_file_2 exists

        test_util::ensure_no_file(&test_file_1);
        cmd::add(vec![], true, false); // add all
        assert!(!test_file_1.exists());
        cmd::commit("delete file 1".to_string(), false);
        let branch_master = match head::current_head()/*  master: test_file_1 not exists, test_file_2 exists */{
            head::Head::Branch(branch) => branch,
            _ => panic!("current head is not branch"),
        };

        /* test 4: switch to branch */
        let result = switch_to(test_branch_2.clone(), false);
        assert!(result.is_ok());
        assert!(status::changes_to_be_staged().is_empty() && status::changes_to_be_committed().is_empty());
        assert!(match head::current_head() {
            head::Head::Branch(branch) => branch == test_branch_2,
            _ => false,
        });
        assert!(test_file_1.exists());
        assert!(!tees_file_2.exists());

        /* test 5: switch to commit */
        let result = switch_to(history_commit.clone(), true);
        assert!(result.is_ok());
        assert!(status::changes_to_be_staged().is_empty() && status::changes_to_be_committed().is_empty());
        assert!(match head::current_head() {
            head::Head::Detached(commit) => commit == history_commit,
            _ => false,
        });
        assert!(test_file_1.exists());
        assert!(tees_file_2.exists());
        assert!(match head::current_head() {
            head::Head::Detached(commit) => commit == history_commit,
            _ => false,
        });

        /* test 6: switch to master */
        let result = switch_to(branch_master.clone(), false);
        assert!(result.is_ok());
        assert!(match head::current_head() {
            head::Head::Branch(branch) => branch == branch_master,
            _ => false,
        });
        assert!(!test_file_1.exists());
        assert!(tees_file_2.exists());
        assert!(status::changes_to_be_staged().is_empty() && status::changes_to_be_committed().is_empty());
    }
}
