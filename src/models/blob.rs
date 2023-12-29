use base64::Engine;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};

use crate::{models::Hash, utils::store};

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
    pub fn new(data: String) -> Blob {
        let mut blob = Blob { hash: "".to_string(), data };
        blob.save();
        blob
    }

    /// 从源文件新建blob对象，但不保存到/objects/中
    pub fn dry_new(data: String) -> Blob {
        let mut blob = Blob { hash: "".to_string(), data };
        let s = store::Store::new();
        let hash: String = s.dry_save(&Blob::encode(blob.data.clone()));
        blob.hash = hash;
        blob
    }

    fn encode(data: String) -> String {
        let mut cmopress_encoder = GzEncoder::new(Vec::new(), Compression::default());
        cmopress_encoder.write_all(data.as_bytes()).unwrap();
        let compressed_data = cmopress_encoder.finish().unwrap();
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(&compressed_data)
    }
    fn decode(encoded: String) -> String {
        let compressed_data = base64::engine::general_purpose::STANDARD_NO_PAD.decode(&encoded).unwrap();
        let mut decompress_decoder = GzDecoder::new(&compressed_data[..]);
        let mut data = String::new();
        decompress_decoder.read_to_string(&mut data).unwrap();
        data
    }

    pub fn load(hash: &String) -> Blob {
        let s = store::Store::new();
        let encoded_data = s.load(hash);
        let data = Blob::decode(encoded_data);
        Blob { hash: hash.clone(), data }
    }

    /// 写入文件
    pub fn save(&mut self) -> Hash {
        let s = store::Store::new();
        let hash: String = s.save(&Blob::encode(self.data.clone()));
        self.hash = hash;
        self.hash.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_content(&self) -> String {
        self.data.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::utils::test;

    #[test]
    fn test_save_and_load() {
        test::setup_with_clean_mit();
        let test_data = "hello world";
        let blob = super::Blob::new(test_data.into());

        let blob2 = super::Blob::load(&blob.hash);
        assert_eq!(blob2.get_hash(), blob.get_hash());
        assert_eq!(blob2.data, test_data);
    }
}
