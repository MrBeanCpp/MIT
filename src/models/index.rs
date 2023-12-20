use crate::models::blob::Blob;
use crate::models::object::Hash;
use crate::utils::util;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// 文件元数据结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetaData {
    pub hash: Hash,                // SHA-1 哈希值
    pub size: u64,                 // 文件大小
    pub created_time: SystemTime,  // 创建时间
    pub modified_time: SystemTime, // 修改时间
    pub mode: String,              // 文件模式
}

impl FileMetaData {
    pub fn new(blob: &Blob, file: &Path) -> FileMetaData {
        let meta = file.metadata().unwrap();
        FileMetaData {
            hash: blob.get_hash(),
            size: meta.len(),
            created_time: meta.created().unwrap(),
            modified_time: meta.modified().unwrap(),
            mode: util::get_file_mode(file),
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
            working_dir: util::get_working_dir().unwrap(),
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
    fn get(&self, path: &Path) -> Option<&FileMetaData> {
        self.entries.get(path)
    }

    pub fn get_hash(&self, file: &Path) -> Option<Hash> {
        Option::from(self.get(file)?.hash.clone())
    }

    pub fn verify_hash(&self, file: &Path, hash: &Hash) -> bool {
        &self.get_hash(file).unwrap_or_default() == hash
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

    /// 与暂存区比较，确定文件自上次add以来是否被编辑（内容不一定修改，还需要算hash）
    pub fn is_modified(&self, file: &Path) -> bool{
        if let Some(self_data) = self.get(file) {
            if let Ok(meta) = file.metadata() {
                let same =  self_data.created_time == meta.created().unwrap_or(SystemTime::now())
                && self_data.modified_time == meta.modified().unwrap_or(SystemTime::now())
                && self_data.size == meta.len();

                return !same;
            } else {
                true
            }
        } else {
            true
        }
    }

    pub fn update(&mut self, path: PathBuf, data: FileMetaData) {
        self.entries.insert(path, data);
    }

    fn load(&mut self) {
    }

    /// 二进制序列化
    pub fn save(&self) {
        //要先转化为相对路径
        let ser = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", ser);
    }

    /** 获取跟踪的文件列表 */
    pub fn get_tracked_files(&self) -> Vec<PathBuf> {
        // XXX 测试版本，有待修改
        let mut files = Vec::new();
        self.entries.keys().for_each(|file| {
            if file.exists() {
                files.push(file.clone());
            }
        });
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::util;
    use std::fs;

    #[test]
    fn test_index() {
        // 示例：获取文件的元数据
        let metadata = fs::metadata(".gitignore").unwrap();
        println!("{:?}", util::format_time(&metadata.created().unwrap()));
        println!("{:?}", util::format_time(&metadata.modified().unwrap()));
        println!("{:?}", metadata.len());
    }

    #[test]
    fn test_save() {
        util::setup_test_with_clean_mit();
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
        index.add(
            PathBuf::from("../src/models/index.rs"),
            FileMetaData::new(
                &Blob::new(Path::new("../src/models/index.rs")),
                Path::new("../src/models/index.rs"),
            ),
        );
        index.save();
    }
}
