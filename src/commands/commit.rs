use crate::head;
use crate::models::{commit, index};

use super::status;

fn no_change() -> bool {
    //todo: move to status.rs
    let change = status::changes_to_be_committed();
    change.new.len() == 0 && change.modified.len() == 0 && change.deleted.len() == 0
}
pub fn commit(message: String, allow_empty: bool) {
    let index = index::Index::new();
    if no_change() && !allow_empty {
        panic!("工作区没有任何改动，不需要提交");
    }

    let current_head = head::current_head();
    let current_commit_hash = head::current_head_commit();

    let mut commit = commit::Commit::new(&index, vec![current_commit_hash.clone()], message.clone());
    let commit_hash = commit.save();
    head::update_head_commit(&commit_hash);

    match current_head {
        head::Head::Branch(branch_name) => {
            println!("commit to [{:?}] message{:?}", branch_name, message)
        }
        head::Head::Detached(commit_hash) => {
            println!("Detached HEAD commit {:?} message{:?}", commit_hash[0..7].to_string(), message)
        }
    }

    println!("commit hash: {:?}", commit_hash);
}

#[cfg(test)]
mod test {
    use crate::utils::util;

    #[test]
    #[should_panic]
    fn test_commit_empty() {
        util::setup_test_with_clean_mit();

        super::commit("".to_string(), false);
    }
}
