use serde::{Deserialize, Serialize};

use crate::store;

use super::{index::Index, object::Hash, tree::Tree};
/*Commit
* git中版本控制的单位。
* 一份Commit中对应一份版Tree，记录了该版本所包含的文件；parent记录本次commit的来源，形成了版本树；
* 此外，Commit中还包含了作者、提交者、提交信息等。
*/
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commit {
    #[serde(skip)]
    hash: Hash,
    author: String,    // unimplemented ignore
    committer: String, // unimplemented ignore
    message: String,
    parent: Vec<Hash>, // parents commit hash
    tree: String,      // tree hash
}

impl Commit {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn get_tree_hash(&self) -> String {
        self.tree.clone()
    }
    pub fn get_tree(&self) -> Tree {
        Tree::load(&self.tree)
    }
    pub fn get_parent_hash(&self) -> Vec<Hash> {
        self.parent.clone()
    }
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
    pub fn get_author(&self) -> String {
        self.author.clone()
    }
    pub fn get_committer(&self) -> String {
        self.committer.clone()
    }

    pub fn new(index: &Index, parent: Vec<Hash>, message: String) -> Commit {
        let mut tree = Tree::new(index);
        let tree_hash = tree.save();
        Commit {
            hash: "".to_string(),
            author: "mit".to_string(),
            committer: "mit-author".to_string(),
            message,
            parent,
            tree: tree_hash,
        }
    }

    pub fn load(hash: &String) -> Commit {
        let s = store::Store::new();
        let commit_data = s.load(hash);
        let mut commit: Commit = serde_json::from_str(&commit_data).unwrap();
        commit.hash = hash.clone();
        commit
    }

    pub fn save(&mut self) -> String {
        // unimplemented!()
        let s = store::Store::new();
        let commit_data = serde_json::to_string_pretty(&self).unwrap();
        let hash = s.save(&commit_data);
        self.hash = hash.clone();
        hash
    }
}

#[cfg(test)]
mod test {
    use crate::utils::util;

    #[test]
    fn test_commit() {
        util::setup_test_with_mit();

        let index = super::Index::new();
        let mut commit = super::Commit::new(
            &index,
            vec!["123".to_string(), "456".to_string()],
            "test".to_string(),
        );
        assert!(commit.hash.len() == 0);

        let hash = commit.save();
        assert!(commit.hash == hash, "commit hash not equal");

        let commit = super::Commit::load(&hash);
        assert!(commit.hash == hash);
        assert!(commit.hash.len() != 0);
        assert!(commit.parent.len() == 2);
        println!("{:?}", commit)
    }
}
