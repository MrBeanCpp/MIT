use sha1::{
    Digest,
    Sha1
};

pub fn calc_hash(data: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash)
}