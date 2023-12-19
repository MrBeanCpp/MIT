use serde::{Deserialize, Serialize};

use crate::store;

use super::{index::Index, object::Hash};
/*Tree
* Tree是一个版本中所有文件的集合。从根目录还是，每个目录是一个Tree，每个文件是一个Blob。Tree之间互相嵌套表示文件的层级关系。
* 每一个Tree对象也是对应到git储存仓库的一个文件，其内容是一个或多个TreeEntry。
*/
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeEntry {
    pub filemode: (String, String), // (type, mode), type: blob or tree; mode: 100644 or 04000
    pub object_hash: Hash,          // blob hash or tree hash
    pub name: String,               // file name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    #[serde(skip)]
    pub hash: Hash,
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(index: &Index) -> Tree {
        Tree {
            hash: "".to_string(),
            entries: Vec::new(),
        }
    }

    pub fn load(hash: &String) -> Tree {
        let s = store::Store::new();
        let tree_data = s.load(hash);
        let mut tree: Tree = serde_json::from_str(&tree_data).unwrap();
        tree.hash = hash.clone();
        tree
    }

    pub fn save(&self) -> String {
        let s = store::Store::new();
        let tree_data = serde_json::to_string(&self).unwrap();
        let hash = s.save(&tree_data);
        hash
    }
}
