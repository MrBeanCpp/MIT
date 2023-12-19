use crate::models::object::Hash;
use crate::store::Store;
use crate::utils::util::calc_hash;
use std::fs;
use std::path::Path;

/**
    Blob
Blob是git中最基本的对象，他储存一份文件的内容，并使用hash作为标识符。
*/
#[derive(Debug, Clone)]
pub struct Blob {
    hash: Hash,
    data: String,
}

impl Blob {
    pub fn new(file: &Path) -> Blob {
        let data = fs::read_to_string(file).unwrap();
        let hash = calc_hash(&data);
        Blob { hash, data }
    }

    pub fn load(hash: &String) -> Blob {
        let s = Store::new();
        let data = s.load(hash);
        Blob {
            hash: hash.clone(),
            data,
        }
    }

    pub fn save(&self) {
        let s = Store::new();
        s.save(&self.data);
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
}
