use std::fs;
use std::path::Path;
use crate::models::object::Hash;
use crate::utils::util::calc_hash;

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
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
}