use super::{index::Index, object::Hash};
/*Commit
* git中版本控制的单位。
* 一份Commit中对应一份版Tree，记录了该版本所包含的文件；parent记录本次commit的来源，形成了版本树；
* 此外，Commit中还包含了作者、提交者、提交信息等。
*/
#[derive(Debug, Clone)]
pub struct Commit {
    hash: Hash,
    author: String,    // unimplemented ignore
    committer: String, // unimplemented ignore
    message: String,
    parent: Vec<Hash>, // parents commit hash
    tree: String,      // tree hash
}

impl Commit {
    pub fn new(index: &Index, parent: Vec<Hash>, message: String) -> Commit {
        unimplemented!()
    }

    pub fn load(hash: &String) -> Commit {
        unimplemented!()
    }

    pub fn save(&self) {
        unimplemented!()
    }
}
