use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use crate::models::object::Hash;

// 文件元数据结构
#[derive(Debug, Clone)]
struct FileMetaData {
    hash: Hash,              // SHA-1 哈希值
    size: u64,                 // 文件大小
    created_time: SystemTime,  // 创建时间
    modified_time: SystemTime, // 修改时间
    mode: String,                 // 文件模式
}

// 索引数据结构
#[derive(Debug, Default)]
struct Index {
    entries: HashMap<PathBuf, FileMetaData>,
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
}