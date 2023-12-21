# MIT: git implementation in Rust

Git in Rust. 用 Rust 编写的简易 Git

## 主要功能

-   支持 git init, git add, git rm, git commit

    -   [x] init
    -   [x] add
    -   [x] rm
    -   [x] commit

- 支持分支 git branch, git checkout

  -   [x] branch
  -   [ ] switch
  -   [ ] restore

  ```bash
  # 撤销未暂存的文件更改（不涉及un trached file)
  git restore path
  git restore . # 全部
  ```

  

-   支持简单的合并 git merge

-   -   [ ] Merge(FF)
