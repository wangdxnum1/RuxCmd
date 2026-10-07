# Skill Manager - 技能管理工具设计文档

## 概述

一个用 Rust 开发的 CLI 工具，用于管理 AI IDE（如 Trae CN、Trae、Cursor）的技能（Skills）。支持安装、更新、列出、卸载技能，并可扩展到多种目标平台。

## 目标

- 提供简洁的命令行界面管理技能
- 支持 GitHub 仓库、本地文件等多种技能源
- 默认支持 Trae CN，可扩展到 Trae、Cursor 等平台
- 配置文件管理，支持自定义平台路径

## 架构

```
┌─────────────────────────────────────────────────────┐
│                     CLI Layer                       │
│   main.rs ──> cli.rs (clap 命令行解析)               │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                   Core Layer                        │
│   config.rs (配置管理)                              │
│   installer.rs (安装/更新/卸载逻辑)                  │
│   utils.rs (工具函数)                               │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                  Abstraction Layer                  │
│   platform::Platform trait (平台适配)                │
│   source::SkillSource trait (技能源)                │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                  Implementation                     │
│   trae_cn.rs / trae.rs / cursor.rs                 │
│   github.rs / local.rs                             │
└─────────────────────────────────────────────────────┘
```

## 核心组件

### 1. CLI 模块 (cli.rs)

使用 `clap` 库解析命令行参数。

支持的命令：
- `install <source>` - 安装技能
- `update <skill_name>` - 更新技能
- `list` - 列出已安装技能
- `uninstall <skill_name>` - 卸载技能
- `config <subcommand>` - 配置管理

### 2. 配置模块 (config.rs)

使用 `toml` 库读写配置文件。

配置文件路径：`~/.skill-manager/config.toml`

配置格式：
```toml
[default]
platform = "trae-cn"

[platforms.trae-cn]
skills_dir = "~/.trae-cn/skills"

[platforms.trae]
skills_dir = "~/.trae/skills"

[platforms.cursor]
skills_dir = "~/.cursor/skills"
```

### 3. 安装模块 (installer.rs)

核心业务逻辑，协调平台适配层和技能源层。

### 4. 平台适配层 (platform/)

定义 `Platform` trait：

```rust
pub trait Platform {
    fn name(&self) -> &str;
    fn skills_dir(&self) -> Result<PathBuf>;
    fn install_skill(&self, skill: &Skill) -> Result<()>;
    fn uninstall_skill(&self, skill_name: &str) -> Result<()>;
    fn list_skills(&self) -> Result<Vec<Skill>>;
}
```

支持的平台：
- Trae CN
- Trae
- Cursor

### 5. 技能源层 (source/)

定义 `SkillSource` trait：

```rust
pub trait SkillSource {
    fn fetch_skill(&self, source: &str) -> Result<Skill>;
    fn update_skill(&self, skill: &Skill) -> Result<Skill>;
}
```

支持的技能源：
- GitHub 仓库（格式：`owner/repo` 或 `owner/repo#branch`）
- 本地目录（格式：`/path/to/skills`）

### 6. 数据结构

```rust
pub struct Skill {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
    pub files: Vec<SkillFile>,
}

pub struct SkillFile {
    pub path: PathBuf,
    pub content: Vec<u8>,
}
```

## 依赖

| 依赖 | 用途 |
|---|---|
| clap | 命令行参数解析 |
| toml | TOML 配置文件读写 |
| serde | 序列化/反序列化 |
| reqwest | HTTP 请求（GitHub API） |
| tar | 解压 tar 包 |
| flate2 | 解压 gzip |
| dirs | 获取用户目录 |
| thiserror | 错误处理 |
| log | 日志 |
| env_logger | 日志实现 |

## 构建配置

遵循项目约束：
- 使用静态 CRT（`-C target-feature=+crt-static`）
- 目标平台：`x86_64-pc-windows-msvc`

## 测试计划

- 单元测试：各模块独立测试
- 集成测试：完整流程测试（安装、更新、卸载）
- 平台测试：针对不同平台的路径和行为测试

## 扩展计划

未来可扩展：
- 支持更多平台（Claude Code、Gemini Code Assist 等）
- 支持技能市场（在线搜索和安装）
- 支持技能版本管理
- 支持技能依赖关系管理