use crate::{
    head,
    models::{commit, index},
};

use super::status;

pub fn commit(message: String, allow_empty: bool) {
    let index = index::Index::new();
    if !allow_empty && status::changes_to_be_committed().is_empty() {
        panic!("工作区没有任何改动，不需要提交");
    }

    let current_head = head::current_head();
    let current_commit_hash = head::current_head_commit();

    let mut commit = {
        if current_commit_hash.is_empty() {
            commit::Commit::new(&index, vec![], message.clone())
        } else {
            commit::Commit::new(&index, vec![current_commit_hash.clone()], message.clone())
        }
    };
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
    use std::path::Path;

    use crate::{commands, head, models, utils::util};

    #[test]
    #[should_panic]
    fn test_commit_empty() {
        util::setup_test_with_clean_mit();

        super::commit("".to_string(), false);
    }

    #[test]
    fn test_commit() {
        util::setup_test_with_clean_mit();
        let test_file = "a.txt";
        let head_one = head::current_head_commit();
        assert!(head_one.is_empty());

        util::ensure_test_file(&Path::new(test_file), "test content".into());
        commands::add::add(vec![], true, false);
        commands::commit::commit("test commit 1".to_string(), true);
        let head_two = head::current_head_commit();
        assert!(head_two.len() > 0);

        let commit = models::commit::Commit::load(&head_two);
        assert!(commit.get_parent_hash().len() == 0);
        assert!(commit.get_message() == "test commit 1");
    }
}
