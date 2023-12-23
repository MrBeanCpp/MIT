use std::path::PathBuf;

use crate::models::object::Hash;

use super::utils::util;

/// 管理.mit仓库的读写
pub struct Store {
    store_path: PathBuf,
}

impl Store {
    pub fn new() -> Store {
        util::check_repo_exist();
        let store_path = util::get_storage_path().unwrap();
        Store { store_path }
    }
    pub fn load(&self, hash: &String) -> String {
        /* 读取文件内容 */
        let mut path = self.store_path.clone();
        path.push("objects");
        path.push(hash);
        match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => panic!("储存库疑似损坏，无法读取文件"),
        }
    }

    pub fn contains(&self, hash: &String) -> bool {
        let mut path = self.store_path.clone();
        path.push("objects");
        path.push(hash);
        path.exists()
    }

    /// 将hash对应的文件内容(主要是blob)还原到file
    pub fn restore_to_file(&self, hash: &Hash, file: &PathBuf) {
        let content = self.load(hash);
        // 保证文件层次存在
        let mut parent = file.clone();
        parent.pop();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::write(file, content).unwrap();
    }

    /** 根据前缀搜索，有歧义时返回 None */
    pub fn search(&self, hash: &String) -> Option<Hash> {
        if hash.is_empty() {
            return None;
        }
        let objects = util::list_files(self.store_path.join("objects").as_path()).unwrap();
        // 转string
        let objects = objects
            .iter()
            .map(|x| x.file_name().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let mut result = None;
        for object in objects {
            if object.starts_with(hash) {
                if result.is_some() {
                    return None;
                }
                result = Some(object);
            }
        }
        match result {
            None => None,
            Some(result) => Some(result),
        }
    }

    pub fn save(&self, content: &String) -> String {
        /* 保存文件内容 */
        let hash = util::calc_hash(content);
        let mut path = self.store_path.clone();
        path.push("objects");
        path.push(&hash);
        println!("Saved to: [{}]", path.display());
        match std::fs::write(path, content) {
            Ok(_) => hash,
            Err(_) => panic!("储存库疑似损坏，无法写入文件"),
        }
    }
}
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_new_success() {
        util::setup_test_with_clean_mit();
        let _ = Store::new();
    }

    #[test]
    #[should_panic]
    fn test_new_fail() {
        util::setup_test_without_mit();
        let _ = Store::new();
    }

    #[test]
    fn test_save_and_load() {
        let _ = util::setup_test_with_clean_mit();
        let store = Store::new();
        let content = "hello world".to_string();
        let hash = store.save(&content);
        let content2 = store.load(&hash);
        assert_eq!(content, content2, "内容不一致");
    }

    #[test]
    fn test_search() {
        util::setup_test_with_clean_mit();
        let hashs = vec!["1234567890".to_string(), "1235467891".to_string(), "4567892".to_string()];
        for hash in hashs.iter() {
            let mut path = util::get_storage_path().unwrap();
            path.push("objects");
            path.push(hash);
            fs::write(path, "hello world").unwrap();
        }
        let store = Store::new();
        assert!(store.search(&"123".to_string()).is_none()); // 有歧义
        assert!(store.search(&"1234".to_string()).is_some()); // 精确
        assert!(store.search(&"4".to_string()).is_some()); // 精确
        assert!(store.search(&"1234567890123".to_string()).is_none()); // 不匹配
    }
}
