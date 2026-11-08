# RuxCmd Desktop 设计说明

**简体中文** | [English](desktop-design.md)

> 状态：**设计已定稿（v1，未开工）**。撰写日期 2026-11-08，基于套件 v0.5.11。
> 技术选型：**Tauri**（Rust 后端 + Web 前端），Windows x64 / MSVC 优先。
> 本文只定义定位、边界与契约，不承诺具体日期；逐轮执行记录见 [迭代状态](iteration-status.md)。

## 0. 已确认决策（2026-11-08，维护者拍板）

| # | 决策 | 说明 |
| --- | --- | --- |
| 1 | **Tauri 2.x 稳定线** | 锁定 major 2，不追 3.0 alpha；选型时不依赖 `releases/latest`（会混入预发布） |
| 2 | **暂不实施代码签名**（2026-11-08 维护者决定） | [代码签名方案](desktop-signing-zh.md) 保留为**未来参考**，不启动申请、不接入 CI、不作为 P2 前置条件。当前状态与 winget 清单一致：未签名、首次运行可能有 SmartScreen 警告；该风险如实告知用户，不掩盖 |
| 3 | **代码放在主仓库顶层 `desktop/`** | 独立 `Cargo.toml`，不加入 `crates/*`；不污染 142 命令的 Verify / 打包门禁。见 §3.1 与 §8 |
| 4 | **UI 采用当前原型为 v1，暂不迭代** | [desktop-prototype.html](desktop-prototype.html) 即 v1 定稿：About / 检查更新 / 设置三视图、三种更新状态、更新进度与回滚提示；后续改动另立版本 |

配套文档：[代码签名方案（暂不实施，留作参考）](desktop-signing-zh.md)（[English](desktop-signing.md)）· [UI 原型](desktop-prototype.html)。

## 1. 定位

**RuxCmd Desktop 是用户唯一需要记住的图形入口：我装了什么、是不是最新、怎么安全地换到新版。**

它服务的是「安装生命周期」，不是命令本身：

- 套件由 142 个独立 exe 组成，安装后在当前系统里几乎不可见——脚本安装只留下 `%LOCALAPPDATA%\RuxCmd\bin\.ruxcmd-install.json` 与一条用户 PATH；Inno 安装只留下「设置 → 应用」里一条卸载记录（图标是 `find.exe`）。没有开始菜单项、没有统一 `--version` 入口。
- [发展方向与见解](roadmap-zh.md) 判断当前最缺的是「分发与可见性」。winget / scoop 是借别人的入口，RuxCmd Desktop 是项目自有的落地页。
- 已有的 [scripts/install.ps1](../scripts/install.ps1) 实现了可靠的更新链路：SHA256 校验 → 解压 → 备份到 `<InstallDir>.old-<版本>` → 原子替换 → PATH/shim 修复 → 失败回滚。**但没有任何「查当前最新版」的能力**，用户必须自己去 GitHub Releases 页面看。

因此 RuxCmd Desktop 补两件事：**存在感**（About）与**版本闭环**（检查更新 + 更新）。

### 1.1 明确不做

| 不做 | 原因 |
| --- | --- |
| 命令启动器 / 内嵌终端 | 142 个 CLI 工具，GUI 点按钮执行没有意义，会误导用户以为主用法是图形界面 |
| 选项 / 别名 / 配置编辑器 | 项目坚持「声明差异、不模拟 POSIX」，没有可配置的运行时状态；加配置即承诺维护 |
| 内嵌文档浏览器 | 双语 docs 已有，GitHub Pages 是 roadmap 4.1 的独立项 |
| 重写下载 / 校验 / 替换逻辑 | `install.ps1` 已实现并通过脚本测试；重写会分裂成第二套逻辑，违背「一处修正处处生效」 |
| 管理 142 个命令的启停 | 架构是独立 exe，无服务、无常驻进程，没有可管对象 |
| 自动后台静默升级 | 更新会替换用户 PATH 下的可执行文件，必须用户知情并显式触发 |

一句话：**它是「安装的前端」，不是「套件的控制台」。**

## 2. 用户与场景

| 用户 | 场景 |
| --- | --- |
| 通过 ZIP 手动解压的用户 | 想知道自己装的是哪个版本、是否落后、怎么升级 |
| 通过 `install.ps1` 安装的用户 | 想一条命令都不敲地完成升级 |
| 通过 Inno EXE 安装的用户 | 想在 Windows 里有一个可找到的 RuxCmd 入口 |
| 从脚本/CI 迁移来的开发者 | 想核对安装溯源（源码提交、哈希、命令数） |

## 3. 架构

```
┌─────────────────────────────────────────────────────────┐
│  WebView (HTML/CSS/JS)                                  │
│  About · 更新检查 · 更新执行 · 设置                     │
│  只负责渲染与交互，不直接访问文件系统/网络/注册表        │
└───────────────────────┬─────────────────────────────────┘
                        │ Tauri IPC (typed commands + events)
┌───────────────────────▼─────────────────────────────────┐
│  Rust 后端 (Tauri commands)                             │
│  ├─ detect.rs   安装状态探测（只读）                    │
│  ├─ release.rs  GitHub Releases 查询（只读）            │
│  ├─ integrity.rs SHA256 / 清单校验（只读）              │
│  ├─ update.rs   更新编排（唯一写路径）                  │
│  └─ model.rs    共享数据结构（serde）                   │
└───────────────────────┬─────────────────────────────────┘
                        │ 复用既有产物，不重新实现
        ┌───────────────┼────────────────┬──────────────────┐
        ▼               ▼                ▼                  ▼
  .ruxcmd-install   Inno 卸载键     GitHub Releases   install.ps1 /
  .json (share)     (HKCU/HKLM)     REST API          Inno setup.exe
```

**关键原则**：Rust 后端是薄编排层。所有写操作最终落到既有脚本/安装包；后端不自己解压 ZIP、不自己改 PATH、不自己生成 shim。

### 3.1 技术选型

- **Tauri 2.x 稳定线**（锁定 major，不追 alpha；Tauri 3 目前为 alpha，GitHub `releases/latest` 会混入预发布，选型时显式指定 2.x）。理由：与套件同为 Rust、单文件分发、无 Node/.NET 运行时依赖、可复用项目既有的 `windows` crate 使用经验。
- 前端用原生 HTML/CSS/JS 或极轻量框架；不引入构建复杂度高的前端栈——本项目是 CLI 工具集，UI 不是主战场。
- 目标平台与套件一致：`x86_64-pc-windows-msvc`。ARM64 延后。
- **独立于套件 workspace**（已决策 3）：放在**主仓库顶层 `desktop/`**，独立 `Cargo.toml`，不加入 `crates/*`，避免污染 142 个命令的 Verify / 打包门禁。
- **版本独立**：Manager 有自己的版本号（如 `0.1.0`），不随套件 0.5.x→0.6.0 联动。About 页同时显示「Manager 版本」与「套件版本」，避免混淆。

## 4. 数据模型

### 4.1 安装状态探测（两种安装方式必须统一）

`detect_installation()` 返回统一结构，来源按优先级探测，冲突时明确报告而不是猜测：

| 来源 | 位置 | 关键字段 |
| --- | --- | --- |
| 脚本安装状态文件 | `<InstallDir>\.ruxcmd-install.json` | `schemaVersion`, `product`, `version`, `target`, `source`, `sha256`, `installedAt`, `installDir`, `binaries[]`, `shims[]`, `backup` |
| Inno 卸载注册表 | `HKCU` / `HKLM` `Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}_is1` | `DisplayVersion`, `InstallLocation`, `DisplayName`, `Publisher`, `UninstallString` |
| 发布清单 | `<InstallDir>\share\manifest.json` | `schemaVersion`, `suiteVersion`, `target`, `rustcVersion`, `sourceCommit`, `dirty`, `binaries[]`, `files[]` |
| 逐文件校验 | `<InstallDir>\share\SHA256SUMS` | `sha256  path` 行 |

本机实测（2026-11-08）：安装方式为 Inno，`DisplayVersion=0.5.11`，`InstallLocation=C:\Users\Tim\AppData\Local\RuxCmd\bin\`——原型以此为真实数据源。

```rust
// 示意，非最终类型
struct InstallInfo {
    method: InstallMethod,          // Script | Inno | Portable | NotInstalled | Unknown
    suite_version: Option<String>,  // "0.5.11"
    install_dir: Option<PathBuf>,
    source_commit: Option<String>,  // 来自 share/manifest.json
    dirty: Option<bool>,
    target: Option<String>,         // "x86_64-pc-windows-msvc"
    binary_count: Option<usize>,    // 142
    shim_count: Option<usize>,      // 16
    checksum_status: ChecksumStatus,// Verified | Mismatch | Missing | NotChecked
    path_on_user_path: Option<bool>,
    conflicts: Vec<String>,         // 两套状态同时存在、版本不一致等
}
```

`NotInstalled` 不等于错误：允许用户在未安装状态下查看 About、检查最新版、并引导首次安装。

### 4.2 发布信息（来自 GitHub REST）

`GET https://api.github.com/repos/wangdxnum1/RuxCmd/releases/latest`

真实返回字段（v0.5.11 实测）：

```json
{
  "tag_name": "v0.5.11",
  "name": "RuxCmd v0.5.11",
  "prerelease": false,
  "draft": false,
  "published_at": "2026-11-07T15:21:54Z",
  "html_url": "https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.11",
  "body": "纯文档与发布状态收口……",
  "assets": [
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip",
      "size": 56136068,
      "digest": "sha256:e90ab189…42226c7045",
      "state": "uploaded" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip.sha256", "size": 108, "digest": "…" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe", "size": 16732287, "digest": "…" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe.sha256", "size": 114, "digest": "…" }
  ]
}
```

要点：

- **四个资产是发布基线**（README / CHANGELOG 已承诺），UI 应校验目标版本四个资产齐全且 `state=uploaded`，缺失则拒绝更新并说明。
- `digest` 字段（`sha256:…`）可与 `.zip.sha256` sidecar 内容做**独立交叉校验**——这是 0.5.8 固化的收尾门禁，UI 正好把它自动化。
- 版本比较用语义化版本（semver），不要字符串比较；`v` 前缀归一化。
- `prerelease` / `draft`：默认只提示稳定版；用户可在设置里开启「接收预发布」，且 UI 必须显式标注预发布。
- 未认证 GitHub API 限流 60 次/小时：必须缓存 + `ETag` / `If-None-Match`（304 复用）+ 手动刷新最短间隔；离线时显示「无法检查」，**绝不显示「已是最新」**。

### 4.3 更新状态机

```
Idle ──check──> Checking ──┬──> UpToDate
                           ├──> UpdateAvailable(version, assets, notes)
                           ├──> Offline / RateLimited (可重试)
                           └──> Error(reason)
UpdateAvailable ──confirm──> Downloading(progress)
    ──> Verifying(SHA256 + sidecar + asset digest 三方比对)
    ──> Installing(调用 install.ps1 / Inno 静默升级)
    ──> Done(version) / RolledBack(reason)
```

任何一步失败都必须给出：失败原因、是否保持了原版本、回滚路径（`<InstallDir>.old-<版本>`）。

## 5. 功能规格

### 5.1 About（只读，零风险）

- 套件版本、Manager 版本、目标平台、Rust 工具链版本
- 安装方式、安装目录、PATH 状态、shim 数量、命令数量
- 发布溯源：`sourceCommit`、`dirty`、逐文件 SHA256 自检结果
- 许可证（MIT）、项目主页、文档、变更记录、兼容性表链接
- 「复制诊断信息」按钮：把上列字段导出为文本，便于用户提 issue

### 5.2 检查更新（只读 + 一次网络请求）

- 显示 当前版本 / 最新版本 / 发布时间 / 发布说明（CHANGELOG 对应节）
- 显示目标资产清单与大小（ZIP 56.1 MB、setup 16.7 MB 量级）
- 明确区分：已是最新 / 有更新 / 无法检查（离线或限流）/ 预发布（需设置开启）
- 「检查更新」为显式动作；启动时不自动联网，或至少可关闭

### 5.3 更新（唯一写操作）

- 二次确认，列出：目标版本、将下载的文件、备份位置、需要关闭的终端
- 按安装方式选择更新路径：
  - 脚本安装 → `pwsh -File install.ps1 -Source <version> -Force`
  - Inno 安装 → 下载 `setup.exe` + 其 `.sha256`，校验后静默升级（`/SILENT` 或直接交给用户运行）
  - 便携解压 → 只提供「打开下载页面」，不自动改写用户手工放置的目录
- 下载进度、校验结果、安装进度分段可见
- 成功后重新探测安装状态并刷新 About；失败给回滚指引

### 5.4 设置（最小集）

- 接收预发布（默认关）
- 启动时自动检查（默认关）
- 检查间隔 / 缓存策略
- 语言（简体中文 / English，与项目双语惯例一致）

## 6. Tauri IPC 契约（命令与事件）

| Command | 方向 | 说明 |
| --- | --- | --- |
| `get_install_info` | 前端→后端 | 只读探测，返回 `InstallInfo` |
| `get_manager_info` | 前端→后端 | Manager 自身版本、构建信息 |
| `check_update(include_prerelease)` | 前端→后端 | 只读，返回 `ReleaseInfo` 或 typed error |
| `verify_installation` | 前端→后端 | 按 `SHA256SUMS` 逐文件校验，返回 `ChecksumStatus` |
| `prepare_update(version)` | 前端→后端 | 下载到临时目录并校验，返回 `PreparedUpdate` |
| `apply_update(id)` | 前端→后端 | **唯一写命令**，需前端二次确认后调用 |
| `cancel_update(id)` | 前端→后端 | 取消下载/清理临时文件 |
| `open_external(url)` | 前端→后端 | 用系统浏览器打开（shell 权限最小化） |
| `copy_diagnostics` | 前端→后端 | 复制诊断文本 |

事件（后端→前端）：`update-progress`（阶段 + 字节数）、`update-stage`、`update-log`、`update-finished`。

**权限约束**（`tauri.conf.json` capabilities）：

- 不启用通用 `fs` 写权限；文件访问全部走后端命令，路径由后端白名单确定。
- `shell` 仅允许执行固定的 `pwsh` / `install.ps1` / `setup.exe`，参数由后端构造，**绝不拼接前端传入的字符串**。
- `http` 限定 `api.github.com`、`github.com`、`objects.githubusercontent.com`。
- 启用 CSP，禁止远程脚本、禁止 `eval`。

## 7. 安全与信任链（重点）

1. **未签名带来的信任链风险（已知、已接受）**：一个「下载并执行新二进制」的程序自身若未签名，首次运行会有 SmartScreen 警告，且用户看到「未知发布者」。[scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml](../scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml) 已声明安装包未签名。**已决策 2：暂不实施签名**——[代码签名方案](desktop-signing-zh.md) 仅作未来参考。因此 P2 不因签名阻塞，但必须在 UI 中如实告知用户「未签名 + 可能有警告」，并提供哈希校验方法让用户自行验证；三方哈希校验（见下一条）仍是强制的，与签名与否无关。签名也不会立刻消除 SmartScreen 警告（声誉机制），详见签名方案 §0。
2. **三方哈希校验**：下载后必须同时核对 ①`.zip.sha256` sidecar 内容、②本地计算的 SHA256、③GitHub asset `digest`；三者一致才允许安装。这把 roadmap 4.2 想做的「已发布 asset digest ↔ 清单哈希独立比对」直接内置到更新器里。
3. **HTTP 只走 TLS**，校验目标域名；不因「检查更新」失败而降级到 http。
4. **不执行下载内容**：下载的是 ZIP / setup.exe，更新动作交给既有脚本；UI 不解析、不加载归档内资源。
5. **最小权限**：默认用户级安装（与 `PrivilegesRequired=lowest` 一致），仅在用户显式选择「所有用户」时才请求提权。
6. **失败不破坏现状**：所有写操作在备份之后进行；校验失败时删除临时文件，不触碰安装目录。

## 8. 与现有发布链的关系

| 项 | 决策 |
| --- | --- |
| 是否进 142 exe 的 ZIP | **暂不**。等形态稳定后单独评估；避免改变已验证的打包清单与 `check-manifests` 门禁 |
| 是否进 scoop / winget 清单 | **暂不**。作为独立可选资产（单独 Release 资产） |
| 是否纳入 `build.ps1 -Task Verify` | **暂不**。主仓库顶层 `desktop/` 配独立 CI，避免拖慢 142 命令的 Verify |
| 代码位置 | **主仓库顶层 `desktop/`**（已决策 3），独立 `Cargo.toml` |
| 版本号 | 独立（如 `0.1.0`），不与套件版本联动 |
| 发布溯源 | Manager 的 About 展示的 `sourceCommit` 指**套件**源码提交，不是 Manager 自身；UI 需标注清楚 |
| 签名 | **暂不实施**；[代码签名方案](desktop-signing-zh.md) 留作参考。当前不改变 Release workflow 的「打包 → 算 `.sha256` → 上传」顺序 |

## 9. 路线（建议分阶段）

| 阶段 | 内容 | 风险 |
| --- | --- | --- |
| P0 原型 | ✅ **v1 已定稿**：[desktop-prototype.html](desktop-prototype.html)，三视图 + 三种更新状态 + 更新进度；维护者确认满意，暂不迭代 | 无 |
| P1 只读 | Tauri 2.x 骨架（主仓库顶层 `desktop/`）+ `detect` + `release` + `integrity`，About / 检查更新可用，**不含任何写操作** | 低 |
| P2 更新 | `prepare_update` / `apply_update` + 进度 + 回滚指引；签名不阻塞，但 UI 需展示「未签名」提示与哈希校验结果 | 中（唯一写路径） |
| P3 分发 | 独立打包、独立 CI、可能纳入 winget / scoop、文档站入口 | 中 |
| P4 增值 | 托盘、更新通知、诊断导出、ARM64 | 低 |

## 10. 验收要点

- 两种安装方式（脚本 / Inno）都能被正确识别并给出一致的版本；未安装时不报错。
- 离线、API 限流、404、资产缺失、哈希不一致、下载中断、安装脚本失败——每一种都有明确文案，且不误报「已是最新」、不破坏现有安装。
- 更新后重新探测，About 显示新版本；失败时旧版本仍可用。
- 所有网络请求域名可在配置中审计；所有写操作可追溯到一条后端命令。

---

*本文随设计演进更新；实现细节与逐轮记录以代码和台账为准。配套 UI 原型见 [desktop-prototype.html](desktop-prototype.html)。*
