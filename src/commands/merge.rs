use clap::builder::Str;

use crate::{
    commands, head,
    models::{commit::Commit, object::Hash},
    store::Store,
    utils::util,
};

enum MergeError {
    NoFastForward,
}

fn check_ff(current: &Hash, target: Hash) -> Result<bool, MergeError> {
    let target_commit = Commit::load(&target);
    // 检查current是否是target的祖先
    if *current == target_commit.get_hash() {
        return Ok(true);
    }
    for parent in target_commit.get_parent_hash() {
        let result = check_ff(current, parent);
        if result.is_ok() {
            return result;
        }
    }
    return Err(MergeError::NoFastForward);
}

/** commit 以fast forward到形式合并到当前分支 */
fn merge_ff(commit_hash: String) -> Result<(), MergeError> {
    // 检查当前分支是否可以fast forward到commit
    let current_commit = head::current_head_commit();
    let check = check_ff(&current_commit, commit_hash.clone());
    if check.is_err() {
        return Err(check.unwrap_err());
    }

    // 执行fast forward
    let head = head::current_head();
    match head {
        head::Head::Branch(branch) => {
            head::update_branch(&branch, &commit_hash.clone());
            commands::restore::restore(vec![], commit_hash.clone(), true, true)
        }
        head::Head::Detached(_) => {
            // 相当于切换到了commit_hash，什么都没有发生
            commands::switch::switch(Some(commit_hash.clone()), None, true);
        }
    }
    unimplemented!();
}

/** merge，暂时只支持fast forward */
pub fn merge(branch: String) {
    let merge_commit = {
        if head::list_local_branches().contains(&branch) {
            // Branch Name, e.g. master
            head::get_branch_head(&branch)
        } else {
            // Commit Hash, e.g. a1b2c3d4
            let store = Store::new();
            let commit = store.search(&branch);
            if commit.is_none() || !util::is_typeof_commit(commit.clone().unwrap()) {
                println!("fatal: 非法的 commit hash: '{}'", branch);
                return;
            }
            commit.unwrap()
        }
    };
    // 暂时只支持fast forward
    let _ = merge_ff(merge_commit);
}
