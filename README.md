# MIT: git implementation in Rust

Git in Rust. 用 Rust 编写的简易 Git

## 主要功能

-   支持 git init, git add, git rm, git commit

    -   [x] init
    -   [x] add
    -   [x] rm
    -   [x] commit

-   支持分支 git branch, git checkout

    -   [x] branch
    -   [ ] switch
    -   [ ] restore
        将选中的文件/路径的文件恢复到--source 制定的版本，默认为 HEAD。不指定区域，默认只操作工作区。指定--staged，操作暂存区。同时指定--staged 和--worktree，操作暂存区和工作区。
        -   目录和通配符会去 suorce 中匹配。不会删除未跟踪的文件。
        -   即，文件在 index 里，不在 source 里，文件会被删除。但是，新建的、未被跟踪的文件不会被删除。但是如果新建里的文件和 source 里的文件名字一样，会被覆盖。

-   支持简单的合并 git merge

-   -   [ ] Merge(FF)
