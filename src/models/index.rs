use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::models::object::Hash;

// 文件元数据结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetaData {
    hash: Hash,              // SHA-1 哈希值
    size: u64,                 // 文件大小
    created_time: SystemTime,  // 创建时间
    modified_time: SystemTime, // 修改时间
    mode: String,                 // 文件模式
}

// 索引数据结构
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Index {
    entries: HashMap<PathBuf, FileMetaData>,
}

impl Index {
    // 创建索引
    pub(crate) fn new() -> Index {
        let mut index = Index {
            entries: HashMap::new(),
        };
        index.load();
        return index;
    }

    // 添加文件
    fn add(&mut self, path: PathBuf, data: FileMetaData) {
        self.entries.insert(path, data);
    }

    // 删除文件
    fn remove(&mut self, path: PathBuf) {
        self.entries.remove(&path);
    }

    // 获取文件元数据
    fn get(&self, path: PathBuf) -> Option<&FileMetaData> {
        self.entries.get(&path)
    }

    // 获取所有文件元数据
    fn get_all(&self) -> &HashMap<PathBuf, FileMetaData> {
        &self.entries
    }
    
    pub fn contains(&self, path: &Path) -> bool {
        self.entries.contains_key(path)
    }

    fn load(&mut self) {

    }

    /// 二进制序列化
    fn save(&self) {
        let ser = serde_json::to_string(&self).unwrap();
        println!("{}", ser);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::utils::util;
    use super::*;

    #[test]
    fn test_index() {
        // 示例：获取文件的元数据
        let metadata = fs::metadata("lines.txt").unwrap();
        println!("{:?}", util::format_time(&metadata.created().unwrap()));
        println!("{:?}", util::format_time(&metadata.modified().unwrap()));
        println!("{:?}", metadata.len());
    }

    #[test]
    fn test_save(){
        let mut index = Index::new();
        let metadata = fs::metadata(".gitignore").unwrap();
        let file_meta_data = FileMetaData {
            hash: "123".to_string(),
            size: metadata.len(),
            created_time: metadata.created().unwrap(),
            modified_time: metadata.modified().unwrap(),
            mode: "100644".to_string(),
        };
        index.add(PathBuf::from(".gitignore"), file_meta_data);
        index.save();
    }
}