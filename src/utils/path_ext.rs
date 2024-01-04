use crate::utils::util;
use std::path::{Path, PathBuf};

/**
Path的扩展 基于util 为了解耦，不要再util中使用PathExt
 */
pub trait PathExt {
    fn to_absolute(&self) -> PathBuf;
    fn to_absolute_workdir(&self) -> PathBuf;
    fn to_relative(&self) -> PathBuf;
    fn to_relative_workdir(&self) -> PathBuf;
    fn is_sub_to(&self, parent: &Path) -> bool;
    fn include_in<T, U>(&self, paths: U) -> bool
    where
        T: AsRef<Path>,
        U: IntoIterator<Item = T>;
}
/*
在 Rust 中，当你调用一个方法时，Rust 会尝试自动解引用和自动引用（auto-deref and auto-ref）来匹配方法签名。
如果有一个为 Path 实现的方法，你可以在 PathBuf、&PathBuf、&&PathBuf 等上调用这个方法，Rust 会自动进行必要的解引用。
 */
impl PathExt for Path {
    /// 转换为绝对路径
    fn to_absolute(&self) -> PathBuf {
        util::get_absolute_path(self)
    }

    /// 转换为绝对路径（from workdir相对路径）
    fn to_absolute_workdir(&self) -> PathBuf {
        util::to_workdir_absolute_path(self)
    }

    /// 转换为相对路径（to cur_dir）
    fn to_relative(&self) -> PathBuf {
        util::get_relative_path(self)
    }

    /// 转换为相对路径（to workdir）
    fn to_relative_workdir(&self) -> PathBuf {
        util::to_workdir_relative_path(self)
    }

    /// 从字符串角度判断path是否是parent的子路径（不检测存在性)
    fn is_sub_to(&self, parent: &Path) -> bool {
        util::is_sub_path(self, parent)
    }

    /// 判断是否在paths中（包括子目录），不检查存在
    fn include_in<T, U>(&self, paths: U) -> bool
    where
        T: AsRef<Path>,
        U: IntoIterator<Item = T>,
    {
        util::include_in_paths(self, paths)
    }
}
