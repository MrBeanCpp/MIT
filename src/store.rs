use std::path::PathBuf;

use super::utils::util;

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
    use super::*;

    #[test]
    fn test_new_success() {
        util::setup_test_with_mit();
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
        let _ = util::setup_test_with_mit();
        let store = Store::new();
        let content = "hello world".to_string();
        let hash = store.save(&content);
        let content2 = store.load(&hash);
        assert_eq!(content, content2, "内容不一致");
    }
}
