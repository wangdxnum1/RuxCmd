# RuxCmd 项目现状与基础设施方向

调查日期：2026-10-06。本文基于当前工作目录（包含未提交文件），不是全量编译通过的声明。

后续更新：维护者允许按常见 Rust 方式重整项目，方案改为根 Cargo workspace，正式 crate 归入 crates/。具体设计见 [基础设施设计](superpowers/specs/2026-10-06-project-infrastructure-design.md)。重新核查时 skill-manager、haozip-ad-killer、rust_test_project 已不在工作目录，当前顶层有 111 个 Cargo 项目；下文的 114 个是首次调查时的记录。

## 项目结构

- 顶层有 114 个包含 Cargo.toml 的目录。
- 其中 111 个属于 Linux 风格的 Windows 命令。维护者已确认第一版只发布这些命令；skill-manager、haozip-ad-killer 独立维护，rust_test_project 排除。
- 多数命令使用 src/main.rs 与 src/cli.rs，分别实现行为和参数解析。
- ps 是独立 workspace，包含 ps-sys、ps-core、ps-bin；实际二进制名称为 ps，不能按 package 名称复制产物。
- Rust edition 2021 与 2024 并存，包版本有 0.1.0 和 1.0.0。
- Windows 依赖包括 windows、多个版本的 windows-sys 和 winapi。网络工具使用 reqwest。
- 根 .cargo/config.toml 配置 MSVC 静态 CRT；仍需实际验证最终产物依赖，不能据此断言所有程序完全静态链接。

## 已确认的基础设施缺口

1. build-all.bat 写死 PROJECT_DIR=D:\Work\rust 和 OUTPUT_DIR=D:\develop\rust-tools，无法直接适配当前检出路径。
2. 批量脚本列出 109 个命令，遗漏 ps 和 pwd。目录名、包名、二进制名也不总是一致。
3. 根目录没有 Cargo workspace、统一 README、工具链版本文件或 .github CI 配置。
4. .gitignore 忽略 Cargo.lock，Git 中没有已跟踪的锁文件。ps 的发布脚本使用 --locked，与缺乏已提交锁文件的状态不匹配。
5. 检索到的自动测试集中于 ps 和 skill-manager。多数命令只有手工测试样例，尚无统一行为测试入口。
6. 发布逻辑主要是复制 exe，没有统一版本化归档、产物清单和校验值。
7. 根目录未发现 LICENSE 文件。ps 声明 MIT，但不能由此推断整个仓库的授权选择。
8. 已有未提交修改和新项目；后续变更必须保留这些工作，避免混入不相关文件。

## 已执行的检查

- 查看目录、所有 Cargo manifest 的关键字段、构建脚本、发布脚本、部分命令源码、测试位置及最近 Git 提交。
- cargo metadata --manifest-path cat/Cargo.toml --no-deps --format-version 1 --offline：通过。
- cargo metadata --manifest-path ps/Cargo.toml --no-deps --format-version 1 --offline：通过。
- 本机 rustc 与 cargo 均为 1.99.0；这只说明当前工具链，不代表项目最低版本要求。
- 未执行全量编译、自动测试或发布包运行验证。既有日志与进度记录不能替代当前验证。

## 待讨论的实施方向

### A. 保留独立项目，统一编排（首次建议，现已调整）

建立一份明确的工具清单，记录 manifest 路径、二进制名称、发布分组及冒烟测试策略。PowerShell 脚本从自身位置解析仓库根路径，提供单工具和全量构建、测试与打包入口。保留现有目录以及 ps 的 workspace。

优点是改动集中，便于先获取全量构建基线；代价是仍要管理各独立项目的锁文件，并显式处理共享构建缓存与产物目录。

### B. 迁移到根 Cargo workspace

使用统一锁文件、target 目录和 Cargo workspace 命令。需迁移 ps 的 workspace 元数据及继承关系，并明确排除实验或独立工具。适合作为后续专项变更，需验证依赖 feature 合并对构建的影响。

### C. 改为一个可执行程序分发多个命令

可改变产物组织，但需调整命令入口与调用方式，涉及业务结构重构，不建议作为基础设施第一阶段。

## 第一阶段拟定验收目标

- 从任意检出路径按 README 的前置条件完成 Windows x64 MSVC 构建。
- 显式覆盖全部选定命令，支持选择单个命令；构建或复制失败时返回非零状态。
- 固定经验证的工具链，提交正式项目锁文件，构建入口使用 --locked。
- 运行现有测试及有界、可重复的行为冒烟测试；持续运行、联网和修改系统的命令应指定各自验证方式。
- 本地与 Windows CI 使用相同入口；输出失败命令及可定位的日志。
- 在构建验证通过后，生成含 exe、使用说明、版本/源码信息、工具清单及 SHA-256 校验信息的 ZIP 包；暂定由维护者手动发起发布。
- 补充根 README、贡献说明、命令兼容性清单以及基础 issue/PR 模板。许可证需由维护者明确选择。

发布范围已确认：第一版只包含 Linux 风格命令。skill-manager 与 haozip-ad-killer 独立维护，rust_test_project 排除。

## 参考

- Cargo workspace：https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo.lock：https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html

基础设施设计已整理为独立文档，尚未进入实现。
