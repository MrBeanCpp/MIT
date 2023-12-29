# MIT: Mini-Git implementation in Rust

Git in Rust. 用 `Rust` 实现的mini `Git`. Called `mit`.

> 旨在简洁、高效且安全

> 学习`Git`的最好方法就是去实现`Git`
> 
## 良好的跨平台支持
-   [x] Windows
-   [x] MacOS
-   [x] Linux (Unix-like...)

## 主要功能
-   支持的输入路径(`pathspec`)：文件路径、目录路径（绝对或相对，包括`.` `./` `../`）


-   支持 `mit init`, `mit add`, `mit rm`, `mit commit`

    -   [x] `init`: 初始化（若仓库已存在，则不执行）- `idempotent`
    -   [x] `add`: 将变更添加至暂存区（包括新建、修改、删除），可指定文件或目录
        -   `-A(all)` : 暂存工作区中的所有文件（从根目录开始）变更（新建√ 修改√ 删除√）
        -   `-u(update)`: 仅对暂存区[`index`]中已跟踪的文件进行操作（新建× 修改√ 删除√）
    -   [x] `rm`: 将文件从暂存区 &| 工作区移除. 
        -    `--cached` : 仅从暂存区移除，取消跟踪
        -    `-r(recursive)`: 递归删除目录，删除目录时必须指定该参数
    -   [x] `commit`
    -   [x] `status`: 显示工作区、暂存区、`HEAD` 的状态，（只包含当前目录）；分为三部分：
        -    **Staged to be committed:** 暂存区与`HEAD`(最后一次`Commit::Tree`)比较，即上次的暂存区
        -    **Unstaged:** 暂存区与工作区比较，未暂存的工作区变更
        -    **Untracked:** 暂存区与工作区比较，从未暂存过的文件（即未跟踪的文件）
    -   [x] `log`

-   支持分支 `mit branch`, `mit switch`, `mit restore`

    -   [x] `branch`
    -   [x] `switch`
            与 `checkout` 不同，`switch` 需要指明`--detach`，才能切换到一个`commit`，否则只能切换分支。
            同时为里简化实现，有任何未提交的修改，都不能切换分支。
    -   [x] `restore`: 回滚文件
        -   将指定路径（可包含目录）的文件恢复到`--source` 指定的版本，可指定操作暂存区 &| 工作区
            - `--source`：可指定`Commit Hash` `HEAD` `Branch Name`
        -   若不指定`--source`，且无`--staged`，则恢复到`HEAD`版本，否则从暂存区[`index`]恢复
        -   若`--staged`和`--worktree`均未指定，则默认恢复到`--worktree`
        -   对于`--source`中不存在的文件，若已跟踪，则删除；否则忽略

-   支持简单的合并 `mit merge` (fast-forward)
-   -   [x] Merge(FF)

## 备注
### 名词释义
-   暂存区：`index` or `stage`，保存下一次`commit`需要的的文件快照
-   工作区：`worktree`，用户直接操作的文件夹
-   工作目录：`working directory` or `repository`，代码仓库的根目录，即`.mit`所在的目录
-   `HEAD`：指向当前`commit`的指针
-   已跟踪：`tracked`，指已经在暂存区[`index`]中的文件（即曾经`add`过的文件）
