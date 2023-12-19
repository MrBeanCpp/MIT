use super::{index::Index, object::Hash};
/*Tree
* Tree是一个版本中所有文件的集合。从根目录还是，每个目录是一个Tree，每个文件是一个Blob。Tree之间互相嵌套表示文件的层级关系。
* 每一个Tree对象也是对应到git储存仓库的一个文件，其内容是一个或多个TreeEntry。
*/
#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub filemode: (String, String), // (type, mode), type: blob or tree; mode: 100644 or 04000
    pub hash: Hash,                 // blob hash or tree hash
    pub name: String,               // file name
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub hash: Hash,
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(index: &Index) -> Tree {
        unimplemented!()
    }

    pub fn load(hash: &String) -> Tree {
        unimplemented!()
    }

    pub fn save(&self) {
        unimplemented!()
    }
}
