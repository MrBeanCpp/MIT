use crate::objects::object::Hash;
#[derive(Debug, Clone)]
pub struct Blob {
    hash: Hash,
    data: String,
}