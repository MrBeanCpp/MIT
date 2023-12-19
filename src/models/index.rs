use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::models::blob::Blob;
use crate::models::object::Hash;
use crate::utils::util::{get_file_mode, get_working_dir};

// 文件元数据结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetaData {
    pub hash: Hash,              // SHA-1 哈希值
    pub size: u64,                 // 文件大小
    pub created_time: SystemTime,  // 创建时间
    pub modified_time: SystemTime, // 修改时间
    pub mode: String,                 // 文件模式
}

impl FileMetaData {
    pub fn new(blob: &Blob, file: &Path) -> FileMetaData {
        let meta = file.metadata().unwrap();
        FileMetaData {
            hash: blob.get_hash(),
            size: meta.len(),
            created_time: meta.created().unwrap(),
            modified_time: meta.modified().unwrap(),
            mode: get_file_mode(file)
        }
    }
}

// 索引数据结构
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Index {
    entries: HashMap<PathBuf, FileMetaData>,
    working_dir: PathBuf,
}

impl Index {
    // 创建索引
    pub(crate) fn new() -> Index {
        let mut index = Index {
            entries: HashMap::new(),
            working_dir: get_working_dir().unwrap()
        };
        index.load();
        return index;
    }

    // 添加文件
    pub fn add(&mut self, path: PathBuf, data: FileMetaData) {
        self.entries.insert(path, data);
    }

    // 删除文件
    pub fn remove(&mut self, path: &Path) {
        self.entries.remove(path);
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

    /// 获取所有已删除的文件
    pub fn get_deleted_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.entries.keys().for_each(|file| {
            if !file.exists() {
                files.push(file.clone());
            }
        });
        files
    }

    pub fn update(&mut self, path: PathBuf, data: FileMetaData) {
        self.entries.insert(path, data);
    }

    fn load(&mut self) {

    }

    /// 二进制序列化
    pub fn save(&self) { //要先转化为相对路径
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
        util::setup_test_with_mit();
        let mut index = Index::new();
        let metadata = fs::metadata("../.gitignore").unwrap();
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