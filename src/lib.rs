// 不使用lib.rs的话，就无法在tests里引用到src中的模块
pub mod objects;
pub mod utils;
pub mod commands;
mod store;