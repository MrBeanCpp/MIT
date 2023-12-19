use crate::head;
use crate::models::{commit, index};
// XXX NOT TESTED
pub fn commit(message: String, allow_enpty: bool) {
    let index = index::Index::new();
    // XXX true 需要替换为 index.is_empty()
    if true && !allow_enpty {
        println!("工作区没有任何改动，不需要提交");
    }

    let current_commit_hash = head::current_head_commit();

    let mut commit = commit::Commit::new(&index, vec![current_commit_hash], message);
    let commit_hash = commit.save();
    head::update_head_commit(&commit_hash);
}
