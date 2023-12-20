use crate::head;
use crate::models::{commit, index};

use super::status;

fn no_change() -> bool {
    let change = status::changes_to_be_committed();
    change.new.len() == 0 && change.modified.len() == 0 && change.deleted.len() == 0
}
pub fn commit(message: String, allow_enpty: bool) {
    let index = index::Index::new();
    if no_change() && !allow_enpty {
        panic!("工作区没有任何改动，不需要提交");
    }

    let current_head = head::current_head();
    let current_commit_hash = head::current_head_commit();

    let mut commit =
        commit::Commit::new(&index, vec![current_commit_hash.clone()], message.clone());
    let commit_hash = commit.save();
    head::update_head_commit(&commit_hash);

    {
        let commit_target = {
            match current_head {
                head::Head::Branch(branch_name) => branch_name,
                head::Head::Detached(commit_hash) => commit_hash[..6].to_string(),
            }
        };
        println!("commit to [{:?}] message{:?}", commit_target, message);
        println!("commit hash: {:?}", commit_hash);
    }
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
