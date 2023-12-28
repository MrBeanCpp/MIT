use base64::Engine;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};

use crate::{
    models::Hash,
    utils::{store, util},
};
use std::{fs, path::Path};

/**Blob<br>
git中最基本的对象，他储存一份文件的内容，并使用hash作为标识符。
*/
#[derive(Debug, Clone)]
pub struct Blob {
    hash: Hash,
    data: String,
}

impl Blob {
    /// 从源文件新建blob对象，并直接保存到/objects/中
    pub fn new(file: &Path) -> Blob {
        let data = fs::read_to_string(file).expect(format!("无法读取文件：{:?}", file).as_str());
        let mut blob = Blob { hash: "".to_string(), data };
        blob.save();
        blob
    }

    pub fn load(hash: &String) -> Blob {
        let s = store::Store::new();
        let encoded = s.load(hash);
        let compressed_data = base64::engine::general_purpose::STANDARD_NO_PAD.decode(&encoded).unwrap();
        let mut decompress_decoder = GzDecoder::new(&compressed_data[..]);
        let mut data = String::new();
        decompress_decoder.read_to_string(&mut data).unwrap();
        Blob { hash: hash.clone(), data }
    }

    /// 写入文件
    pub fn save(&mut self) -> Hash {
        let s = store::Store::new();
        let mut cmopress_encoder = GzEncoder::new(Vec::new(), Compression::default());
        cmopress_encoder.write_all(self.data.as_bytes()).unwrap();
        let compressed_data = cmopress_encoder.finish().unwrap();
        let encoded_data = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&compressed_data);
        let hash: String = s.save(&encoded_data);
        self.hash = hash;
        self.hash.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::utils::test_util;

    #[test]
    fn test_save_and_load() {
        test_util::setup_test_with_clean_mit();
        let test_data = "hello world";
        test_util::ensure_test_file(&PathBuf::from("a.txt"), Some(test_data));
        let blob = super::Blob::new(&PathBuf::from("a.txt"));

        let blob2 = super::Blob::load(&blob.hash);
        assert_eq!(blob2.get_hash(), blob.get_hash());
        assert_eq!(blob2.data, test_data);
    }
}
