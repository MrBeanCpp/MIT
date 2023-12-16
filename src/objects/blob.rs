use crate::objects::object::Hash;
/*Blob
* Blob是git中最基本的对象，他储存一份文件的内容，并使用hash作为标识符。
*/
#[derive(Debug, Clone)]
pub struct Blob {
    hash: Hash,
    data: String,
}
