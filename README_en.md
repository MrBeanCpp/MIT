<h1 align="center">
 MIT: Mini-Git implementation in Rust
</h1>

 **[中文文档](./README.md)** | <u>English</u>

[Project Link](https://github.com/MrBeanCpp/MIT)

Git in Rust. A mini Git implementation called`mit`, implemented in `Rust`.

> Designed to be concise, readable, efficient, and secure.
>
> The best way to learn Git is to implement Git.
> 
> This project aims to provide a `Git` implementation that even a second-grader can understand.
> 
> `// rm -rf rigid design patterns & complex repository architecture`
> 

## Cross-Platform Support
-   [x] Windows
-   [x] MacOS
-   [x] Linux (Unix-like...)

## Key Features
-   Supports input paths (pathspec): file paths, directory paths (absolute or relative, including `.`, `./`, `../`)

-   Supports `mit init`, `mit add`, `mit rm`, `mit commit`

    -   [x] `init`: Initialize (does nothing if the repository already exists) - `idempotent`
    -   [x] `add`:  Add changes to the staging area (including new, modified, deleted), can specify files or directories
        -   `-A(all)` : Stage all changes in the working directory (from the root) (new✅ modified✅ deleted✅)
        -   `-u(update)`:  Operate only on tracked files in the staging area [`index`] (new❌ modified✅ deleted✅)
    -   [x] `rm`: Remove files from the staging area & working directory 
        -    `--cached` : Remove only from the staging area, untrack
        -    `-r(recursive)`: Recursively delete directories, must specify this parameter when deleting directories
    -   [x] `commit`
    -   [x] `status`: Display the status of the working directory, staging area, and `HEAD` (only for the current directory); divided into three parts:
        -    **Staged to be committed:**  Changes staged in the staging area compared to `HEAD` (last `Commit::Tree`), i.e., the last staging area
        -    **Unstaged:** Changes in the working directory not staged in the staging area
        -    **Untracked:** Files in the working directory not staged or tracked before
    -   [x] `log`

-   Supports branches`mit branch`, `mit switch`, `mit restore`

    -   [x] `branch`
    -   [x] `switch`
            Unlike `checkout`, `switch` requires specifying `--detach` to switch to a `commit`, otherwise, it can only switch branches.
    -   [x] `restore`: Rollback files
        -   Restore files at the specified path (including directories) to the version specified by `--source`, can specify staging area & working directory
            - `--source`： Can specify `Commit Hash`, `HEAD`, or `Branch Name`
        -   If `--source` is not specified and neither `--staged` nor `--worktree` is specified, restore to the `HEAD` version, otherwise, restore from the staging area [`index`]
        -   If neither `--staged` nor `--worktree` is specified, default to restore to `--worktree`
        -   For files not present in `--source`, if tracked, delete; otherwise, ignore

-   Supports simple merging `mit merge` (fast-forward)
    -   [x] Merge(FF)

## Notes
### ⚠️Testing requires single-threading
⚠️ Note: To avoid conflicts, please use `--test-threads=1` when executing tests.

For example:`cargo test -- --test-threads=1`

This is because testing involves IO on the same folder.

### Term Definitions
-   Staging area: `index` or `stage`, stores file snapshots needed for the next `commit`
-   Working directory: `worktree`, the folder directly manipulated by the user
-   Repository: `working directory` or `repository`, the root directory of the code repository, where `.mit` is located
-   `HEAD`：Points to the current`commit`
-   Tracked：`tracked`，files already in the staging area [`index`](i.e., files that have been `add`-ed)

### Introductory Video
[【Mit】Rust implementation of Mini-Git - System Software Development Practice Final Report_Bilibili](https://www.bilibili.com/video/BV1p64y1E78W/)
