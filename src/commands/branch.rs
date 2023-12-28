use colored::Colorize;

use crate::{
    models::*,
    utils::{head, store, util},
};

// branch error
enum BranchErr {
    BranchExist,
    InvalidObject,

    BranchNoExist,
    BranchCheckedOut,
}
// 从分支名、commit hash中搜索commit
fn search_hash(commit_hash: Hash) -> Option<Hash> {
    // 分支名
    if head::list_local_branches().contains(&commit_hash) {
        let commit_hash = head::get_branch_head(&commit_hash);
        return Some(commit_hash);
    }
    // commit hash
    let store = store::Store::new();
    let commit = store.search(&commit_hash);
    commit
}

fn create_branch(branch_name: String, _base_commit: Hash) -> Result<(), BranchErr> {
    // 找到正确的base_commit_hash
    let base_commit = search_hash(_base_commit.clone());
    if base_commit.is_none() || util::check_object_type(base_commit.clone().unwrap()) != util::ObjectType::Commit {
        println!("fatal: 非法的 commit: '{}'", _base_commit);
        return Err(BranchErr::InvalidObject);
    }

    let base_commit = Commit::load(&base_commit.unwrap());

    let exist_branches = head::list_local_branches();
    if exist_branches.contains(&branch_name) {
        println!("fatal: 分支 '{}' 已存在", branch_name);
        return Err(BranchErr::BranchExist);
    }

    head::update_branch(&branch_name, &base_commit.get_hash());
    Ok(())
}

fn delete_branch(branch_name: String) -> Result<(), BranchErr> {
    let branches = head::list_local_branches();
    if !branches.contains(&branch_name) {
        println!("error: 分支 '{}' 不存在", branch_name);
        return Err(BranchErr::BranchNoExist);
    }

    // 仅在当前分支为删除分支时，不允许删除（在历史commit上允许删除）
    let current_branch = match head::current_head() {
        head::Head::Branch(branch_name) => branch_name,
        _ => "".to_string(),
    };
    if current_branch == branch_name {
        println!("error: 不能删除当前所在分支 {:?}", branch_name);
        return Err(BranchErr::BranchCheckedOut);
    }

    head::delete_branch(&branch_name); // 删除refs/heads/branch_name，不删除任何commit
    Ok(())
}

fn show_current_branch() {
    println!("show_current_branch");
    let head = head::current_head();
    match head {
        head::Head::Branch(branch_name) => println!("{}", branch_name),
        _ => (), // do nothing
    }
}

fn list_branches() {
    println!("list_branches");
    let branches = head::list_local_branches();
    match head::current_head() {
        head::Head::Branch(branch_name) => {
            println!("* {}", branch_name.green());
            for branch in branches {
                if branch != branch_name {
                    println!("  {}", branch);
                }
            }
        }
        head::Head::Detached(commit_hash) => {
            println!("* (HEAD detached at {}) {}", commit_hash.green(), commit_hash[0..7].green());
            for branch in branches {
                println!("  {}", branch);
            }
        }
    }
}

pub fn branch(
    new_branch: Option<String>,
    commit_hash: Option<Hash>,
    list: bool,
    delete: Option<String>,
    show_current: bool,
) {
    if new_branch.is_some() {
        let basic_commit = if commit_hash.is_some() {
            commit_hash.unwrap()
        } else {
            head::current_head_commit() // 默认使用当前commit
        };
        let _ = create_branch(new_branch.unwrap(), basic_commit);
    } else if delete.is_some() {
        let _ = delete_branch(delete.unwrap());
    } else if show_current {
        show_current_branch();
    } else if list {
        // 兜底list
        list_branches();
    } else {
        panic!("should not reach here")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{commands, utils::util::test_util};
    #[test]
    fn test_create_branch() {
        test_util::setup_test_with_clean_mit();

        // no commit: invalid object
        let result = create_branch("test_branch".to_string(), head::current_head_commit());
        assert!(result.is_err());
        assert!(match result.unwrap_err() {
            BranchErr::InvalidObject => true,
            _ => false,
        });
        assert!(head::list_local_branches().is_empty());

        commands::commit::commit("test commit 1".to_string(), true);
        let commit_hash_one = head::current_head_commit();
        commands::commit::commit("test commit 2".to_string(), true);
        let commit_hash_two = head::current_head_commit();

        // success, use part of commit hash
        let new_branch_one = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let result = create_branch(new_branch_one.clone(), commit_hash_one[0..7].to_string());
        assert!(result.is_ok());
        assert!(head::list_local_branches().contains(&new_branch_one), "new branch not in list");
        assert!(head::get_branch_head(&new_branch_one) == commit_hash_one, "new branch head error");

        // branch exist
        let result = create_branch(new_branch_one.clone(), commit_hash_two.clone());
        assert!(result.is_err());
        assert!(match result.unwrap_err() {
            BranchErr::BranchExist => true,
            _ => false,
        });

        // use branch name as commit hash, success
        let new_branch_two = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let result = create_branch(new_branch_two.clone(), new_branch_one.clone());
        assert!(result.is_ok());
        assert!(head::list_local_branches().contains(&new_branch_two), "new branch not in list");
        assert!(head::get_branch_head(&new_branch_two) == commit_hash_one, "new branch head error");
    }

    #[test]
    fn test_delete_branch() {
        test_util::setup_test_with_clean_mit();

        // no commit: invalid object
        let result = delete_branch("test_branch".to_string());
        assert!(result.is_err());
        assert!(match result.unwrap_err() {
            BranchErr::BranchNoExist => true,
            _ => false,
        });
        assert!(head::list_local_branches().is_empty());

        commands::commit::commit("test commit 1".to_string(), true);
        let commit_hash = head::current_head_commit();

        // success
        let new_branch = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let result = create_branch(new_branch.clone(), commit_hash.clone());
        assert!(result.is_ok());
        assert!(head::list_local_branches().contains(&new_branch), "new branch not in list");
        assert!(head::get_branch_head(&new_branch) == commit_hash, "new branch head error");

        // branch exist
        let result = delete_branch(new_branch.clone());
        assert!(result.is_ok());
        assert!(!head::list_local_branches().contains(&new_branch), "new branch not in list");
    }
}
