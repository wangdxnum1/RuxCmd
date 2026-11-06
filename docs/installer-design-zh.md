# RuxCmd 安装包方案（方案 A：单个 EXE，Inno Setup）

状态：已收口，进入实现。方案 A（单 EXE、Inno Setup 7.1.1、放弃 MSI）已获用户确认；用户已明确接受 Inno Setup 7 的授权条款，并同意把工具链约束放宽到 7.x（仍拒绝 5/6/8）。
本轮只改仓库文件与脚本：没有在本机安装新的 Inno Setup，没有运行安装器/卸载器，没有修改 PATH 或注册表，没有删除任何已有构建产物。

## 0. 结论

- 只发**一个 `ruxcmd-<version>-x86_64-pc-windows-msvc-setup.exe`**，由 Inno Setup 7.1.1 编译；不产出 MSI。
- 首次安装用 Inno 原生能力：启动时选择“仅为当前用户 / 为所有用户”（`PrivilegesRequiredOverridesAllowed=dialog`），并显示目录页让用户自选绝对路径。
- 升级/重装复用同一个 `AppId`，自动沿用旧的安装模式与旧目录、跳过模式与目录页、原地覆盖；同版本重装也覆盖；降级拒绝；升级时改目录被拒绝。
- PATH 只追加一条（用户级写 HKCU，机器级写 HKLM），卸载时精确移除这一条。
- `rux-` shim 在打包期按 ZIP 的 `manifest.json` 生成，作为普通文件安装；旧 shim 在升级时被清理，卸载时兜底清理。
- 安装包只打包现有发布 ZIP 的产物，不维护第二份 exe 清单；打包后再次校验 142 个 exe 的版本资源与安装包自身的版本资源。
- 版本从 `workspace.metadata.ruxcmd.version` 单点取值，安装包与 ZIP `manifest.json`、142 个 exe 版本资源必须一致。
- 未签名，SmartScreen/UAC 的实际表现见 §7。

放弃 MSI 的代价：没有 Windows Installer 事务回滚、没有企业 `msiexec /x`、没有组策略分发。换来的是单文件、无 WiX 依赖、无 MSI/引导器双身份、安装与卸载逻辑全部可见在 Inno 脚本里。

## 1. 参考 niuma-timer 的结论（只读调研，保留作为背景）

[niuma-timer](file:///C:/Users/Tim/Work/my-projects/niuma-timer) 是 Tauri v2 GUI 应用，用 Tauri Bundler 同时产出 NSIS EXE 与 WiX v3 MSI：

- NSIS 固定 `installMode: currentUser` → `%LOCALAPPDATA%`，免 UAC，**没有目录页**。
- MSI 固定 `ALLUSERS=1` + `INSTALLDIR` 在 `ProgramFiles64Folder` + `WixUI_InstallDir`，机器级、要 UAC、**有目录页**；没有 Environment 表，因此**完全不处理 PATH**。
- 升级靠 MSI `Upgrade` 表（UpgradeCode 稳定、`Attributes=257` 允许同版本升级）；NSIS 检测到旧安装会先卸载。
- CI 的可用经验：版本双处校验、固定版本工具安装、按版本号收集产物并 fail-closed、草稿 → 校验 → 公开；安装包未做 Authenticode 签名（只有 updater 的 minisign `.sig`）。

不能照搬的原因：Tauri Bundler 面向单个 GUI 应用，承载不了 142 个 exe 的 CLI 套件与 PATH/shim 需求；它的 NSIS 与 MSI 是两套身份，同时使用会留下两套安装记录；它也没有“用户级 + 自选目录”的组合。可借鉴的只有版本纪律与 CI/发布结构。

## 2. 首次安装：范围与目录

### 2.1 行为

`[Setup]` 的关键设置（见 [RuxCmd.iss](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/RuxCmd.iss)）：

```ini
DefaultDirName={code:GetDefaultDir}
DisableDirPage=auto
UsePreviousAppDir=yes
UsePreviousPrivileges=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
```

首次安装的流程：

1. 启动时 Inno 弹出安装模式选择：`Install for me only` / `Install for all users`（可选择不再询问）。
2. 选“仅当前用户”→ 不请求提权；选“所有用户”→ 请求 UAC 提权。
3. 显示目录页，默认目录随模式变化，用户可以改成任意可写的绝对路径。
4. 目录页离开时即时校验（`NextButtonClick`），安装开始前再校验一次（`PrepareToInstall`）。

| 选择 | 权限 | 默认目录 | PATH 写入 | 卸载注册表项 |
| --- | --- | --- | --- | --- |
| 仅当前用户 | 免 UAC | `%LOCALAPPDATA%\RuxCmd\bin` | HKCU `Environment\Path` | `HKCU\...\Uninstall\{06BD...}_is1` |
| 所有用户 | UAC（未知发布者） | `%ProgramFiles%\RuxCmd\bin` | HKLM `...\Session Manager\Environment\Path` | `HKLM\...\Uninstall\{06BD...}_is1` |

目录合法性规则（`IsValidInstallDirectory`）：

- 拒绝空目录、盘符根目录（如 `D:\`）。
- 拒绝含 `;`、`"`、`%`、CR/LF 的目录：分号会破坏 PATH 分段，引号会破坏带空格路径的引用，`%` 会被注册表当成环境变量展开。
- 非管理员模式拒绝 `Program Files`、`Program Files (x86)`、`Windows`、`System32` 及其子目录，提示改用“所有用户”或换目录（避免装到一半才因权限失败）。

### 2.2 默认目录的来源

`DefaultDirName={code:GetDefaultDir}`：管理员模式返回 `{commonpf}\RuxCmd\bin`，否则 `{localappdata}\RuxCmd\bin`。`DisableDirPage=auto` 只在“检测到同一应用已安装”时隐藏目录页；首次安装始终显示目录页。

## 3. 升级、同版本重装与降级

### 3.1 稳定标识

| 用途 | 取值 |
| --- | --- |
| Inno `AppId` | `{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}`（脚本写作 `{{06BD...}`，首个 `{` 为转义） |
| 卸载键 | `HKCU` 或 `HKLM` 的 `Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD...}_is1` |

`AppId` 永久不变；它是升级身份，也是 `install.ps1` 互斥检测的依据。Inno 判断“同一应用”要求 `AppId`、安装模式（管理/非管理）、32/64 位模式三者一致——我们的安装模式由 `UsePreviousPrivileges` 沿用旧值，64 位由 `ArchitecturesInstallIn64BitMode=x64compatible` 固定。

### 3.2 已经安装时的行为

| 机制 | 效果 |
| --- | --- |
| `UsePreviousPrivileges=yes`（默认） | 已安装过则沿用上次的管理/非管理模式，不再弹模式选择 |
| `UsePreviousAppDir=yes`（默认） | 从卸载键的 `Inno Setup: AppPath` 读回旧目录 |
| `DisableDirPage=auto`（默认） | 检测到已安装时不显示目录页，直接使用旧目录 |
| `[Files] ... Flags: ignoreversion` | 同版本重装也覆盖写入，不做“文件更新”判断 |
| `InitializeSetup` 读 `DisplayVersion` | 旧版本数字更高 → 弹窗提示并中止（拒绝降级） |
| `PrepareToInstall` 比对旧目录 | 旧目录与目标目录不同（含 `/DIR=` 指定）→ 中止，要求先卸载 |

各场景的实际结果：

| 场景 | 行为 |
| --- | --- |
| 同版本重装 | 不提问，复用旧目录，覆盖 exe/shim/share；PATH 检测到已存在则不重复追加 |
| 跨版本升级（新 > 旧） | 同上；旧版多出的 shim 被 `[InstallDelete]` 的 `rux-*.cmd` 通配删除后再写新集合 |
| 升级时改目录 | 拒绝，提示先卸载；不支持“搬家式升级” |
| 降级（新 < 旧） | 拒绝，提示先卸载更高版本（`0.4.4-beta.1` 这类预发布按数字段比较，忽略后缀） |
| 换安装范围（用户↔机器） | 不支持在升级中切换；先卸载再装 |
| 卸载记录损坏/读不到旧目录 | 中止并提示先卸载，不猜目录 |

关于旧 exe：Inno 的卸载日志是**追加式**的。升级时旧版本里已经被删除命令的 exe 不会立即从磁盘消失，但它们仍在卸载日志里，最终卸载时会被删除。这是 Inno 的既定行为，文档如实说明，不在升级时按通配删除 `*.exe`（避免误删用户放进安装目录的文件）。

### 3.3 PATH 的追加与移除

首次安装的成功安装阶段（`CurStepChanged(ssPostInstall)`）执行 `AddPathEntry`：

1. 管理员模式读写 HKLM 的 `SYSTEM\CurrentControlSet\Control\Session Manager\Environment\Path`；否则读写 HKCU 的 `Environment\Path`。
2. 读取现有值（不存在就视为空），按 `;` 分段、忽略大小写、去掉尾部反斜杠后比较。
3. 已存在同一目录 → 不写；否则追加为 `现有值;{app}`（原值为空时直接写 `{app}`，不产生前置分号）。
4. 以 `REG_EXPAND_SZ` 写入（机器级 PATH 默认就是该类型）。

卸载（`CurUninstallStepChanged(usPostUninstall)`）执行 `RemovePathEntryFromEnvironment`：读取同一根键的 PATH，按段精确移除与 `{app}` 相同的项，其余项原样保留；移除后为空则删除该值。不做通配删除、不碰其他项，用户安装后自行追加的 PATH 项不会被覆盖。

写入代码而不放在 `[Registry]` 的原因：`{olddata}` 在值不存在时会产生前置 `;`；而且 Inno 的注册表卸载逻辑会“恢复安装前的旧值”，会丢掉用户在安装之后新增的 PATH 项。

`ChangesEnvironment=yes` 让安装结束时广播环境变更，新开的控制台即可用新 PATH（已开的控制台需要重开）。

### 3.4 shim

- 17 个冲突名：`find sort where cat ls cp mv rm echo pwd ps kill sleep tee clear curl wget`；与 `install.ps1` 共享同一份策略（`Get-RuxCmdPrefixShimCommands` / `Get-RuxCmdShimContent`）。套件不含 `where.exe`，因此实际生成 **16 个** `rux-*.cmd`。
- 内容：`@echo off` + `"%~dp0<command>.exe" %*`，UTF-8 无 BOM，CRLF。
- 打包期按 ZIP `manifest.json` 的 binaries 生成，只对 ZIP 中存在的命令生成；安装包把它们当普通文件装进 `{app}`。
- 升级：`[InstallDelete]` 先删 `{app}\rux-*.cmd`，再写新集合，旧版多出的 shim 不会残留。
- 卸载：`[UninstallDelete]` 再删一次 `rux-*.cmd` 兜底（跨多个版本的卸载日志去重），并删除 `{app}\share`。

## 4. 卸载

| 对象 | 处理方式 |
| --- | --- |
| 142 个 exe、`unins000.exe/.dat` | Inno 卸载日志（含历史追加记录）逐个删除 |
| `rux-*.cmd` | 当前日志 + `[UninstallDelete]` 通配兜底 |
| `share`（README、README-zh、CHANGELOG、compatibility.md、`manifest.json`、`SHA256SUMS`） | `[UninstallDelete] filesandordirs` |
| PATH 中的安装目录项 | `usPostUninstall` 精确移除（只移除这一项） |
| 卸载注册表项 | Inno 自动删除 |
| 用户自己的配置/其他文件 | 不管理、不删除 |

- 卸载权限与安装一致：用户级卸载免 UAC；机器级卸载要 UAC。
- 无事务回滚：Inno 不是事务型安装器。安装中途失败时可能留下部分文件；文档给出的恢复动作是“重跑同一安装包（覆盖安装），或卸载后重装上一版”。升级过程中被中断同理。
- 目录为空后由 `[UninstallDelete] dirifempty` 尝试删除；若卸载器自身文件尚未清空而留下空目录，属正常，可直接手工删除。

## 5. 与 `install.ps1` 的共存

- `install.ps1` 的默认目录已改为 `%LOCALAPPDATA%\RuxCmd\bin`，与安装包“仅当前用户”默认一致；`-InstallDir` 仍可指定任意目录。
- 互斥是双向的、只读的：
  - `install.ps1` 启动时检查 HKCU/HKLM（含 WOW6432Node）的 `...\Uninstall\{06BD...}_is1`，存在则拒绝执行并提示从“设置 → 应用”卸载。测试 seam `RUXCMD_INSTALL_USER_PATH_FILE` 存在时跳过该检查，保证测试不碰真实 profile。
  - 安装器在 `PrepareToInstall` 扫描 HKCU 的 PATH 分段，发现某段的目录里有 `.ruxcmd-install.json`（`install.ps1` 的状态文件）就中止，并给出卸载命令。
- 旧目录 `C:\Users\Tim\Develop\RuxCmd\bin` 的脚本版**不自动迁移**：先 `pwsh -File scripts/install.ps1 -Uninstall -RemoveInstallDir -InstallDir "<旧目录>"`，再运行安装包。自动搬 142 个文件 + PATH + shim 的失败面不值得。

## 6. 工具链与 CI

### 6.1 固定版本与支持区间

- **Inno Setup 7.1.1**（用户已确认使用该版本并接受 Inno Setup 7 的授权条款），版本与下载地址记录在 [toolchain.json](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/toolchain.json)：
  `https://github.com/jrsoftware/issrc/releases/download/is-7_1_1/innosetup-7.1.1-x64.exe`（x64 安装器，安装到 `%ProgramFiles%\Inno Setup 7`）。
- 版本约束：`toolchain.json` 的 `supportedMajorVersion=7` 定义支持区间为 **7.x**。脚本会从注册表读出实际 `ISCC` 版本并打印；**5.x/6.x/8.x 明确拒绝**，7.x 内的任意小版本（如 7.0.2）允许，便于本机已有 7 系工具链直接使用。`innoSetupVersion` 只用于“自动安装时下载哪个版本”。
- 7.1.1 编译出的安装包独立分发，不依赖 Inno 运行时；Inno 的授权变化只影响**使用编译器的一方**，不影响产物的分发与运行。Inno 7 下载页要求商业使用者购买许可（见 jrsoftware 的 Commercial Licenses 页面），本项目按用户决定使用 7.1.1。
- 未配置商业许可密钥时，`ISCC 7.1.1` 的编译横幅会打印 `Non-commercial use only`。本轮本机编译即处于该状态（用户已接受 7.x 条款）；该横幅不写入产物、不改变安装包行为。若 RuxCmd 的发行属于商业使用，需要按 jrsoftware 的许可页面购买并在构建机上配置密钥。
- **Inno 7 改了卸载注册表值的名字**：Inno 6 写作 `Inno Setup: AppPath`，7.x 写作 `Inno Setup: App Path`（带空格）。升级/同版本重装时读取旧目录必须两个名字都尝试，否则会以“已安装但目录无法读取”中止（本轮实测发现并修复）。
- 只需要 `ISCC.exe` 命令行编译器。定位顺序：`PATH` → `%ProgramFiles(x86)%\Inno Setup 6` → `%ProgramFiles%\Inno Setup 6` → 同名 `Inno Setup 7` 目录（x86/x64 安装器各用其一）；也可用 `-Iscc <path>` 显式指定。定位时会优先返回属于支持区间的编译器。
- **版本检测不能读 `ISCC.exe` 的文件版本**：它没有版本资源，`ProductVersion` 恒为 `0.0.0.0`（这正是首次 CI 失败的原因）。版本从卸载注册表 `...\Uninstall\Inno Setup 7_is1`（或 `6_is1`）的 `DisplayVersion` 读取，并与 `InstallLocation` 目录匹配。
- CI 与本机共用 [install-inno.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/install-inno.ps1)：已装 7.x 则跳过并打印实际版本，否则下载 7.1.1 官方安装器静默安装并复验。**7 系静默安装参数与 6 系相同**：`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-`；Inno 的安装器没有单独的 EULA 接受开关，`/VERYSILENT` 不显示许可页，接受条款属于使用者（本项目已由用户确认）。
- `ISCC` 不在 `PATH` 不需要新开会话或加环境变量：脚本按绝对路径调用（新进程会读到已安装的文件）；`PATH` 条目只影响在终端直接输入 `iscc`。

### 6.2 打包脚本

[package-installer.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/package-installer.ps1) 只吃现有 ZIP：

1. 校验 ZIP 存在、`.sha256` 校验和匹配。
2. 从 ZIP 读 `manifest.json`：`schemaVersion=1`、`suiteVersion`、`target`、`sourceCommit` 必须与当前工作区一致；`dirty` 只告警。
3. 断言 manifest binaries 与 `cargo metadata` 得到的二进制名称集合完全一致（142 个，不维护第二份清单）。
4. 解压到本地临时目录，逐文件校验 SHA256，再对每个 exe 跑 `Assert-BinaryVersionInfo`（产品名、产品版本、文件版本、公司、版权、描述、Build/Prerelease 标志）。
5. 按 manifest 生成 17 个 shim 中实际存在的那些。
6. 生成 `build-info.iss`（版本、数字版本、产品名、公司、版权、输出路径）与 `generated-files.iss`（`[Files]` 列表），并断言 `Source:` 行数 = binaries + shims + docs + 2（`manifest.json`、`SHA256SUMS`）。
7. 调用版本落在支持区间内的 `ISCC.exe`（打印实际版本）编译，校验安装包自身的版本资源，复制到 `dist/`，写 `.sha256`。
8. 失败时只清理本次新建的产物与自己的临时目录；不覆盖已存在的安装包。

产物：`dist/ruxcmd-<version>-x86_64-pc-windows-msvc-setup.exe` + `.sha256`。

安装包内文件布局：exe 与 shim → `{app}`；四份文档 + `manifest.json` + `SHA256SUMS` → `{app}\share`。

关于“安装包内 exe 数量校验”：脚本校验的是编译进安装包的精确 payload（同一批解压、逐文件 SHA256、逐个版本资源），以及 `[Files]` 条目数量；在不安装、不借助第三方解包工具的前提下无法直接枚举 setup 内部文件，端到端安装后的计数属于 §9 的授权验证。

### 6.3 CI 增量（在现有流水线上加，不重建二进制）

`ci.yml` 在现有 `dist/*.zip` 步骤之后增加：

1. 读取 `toolchain.json`，下载 7.1.1 安装器并静默安装（`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-`），再读注册表 `DisplayVersion` 复验 `ISCC.exe` 属于 7.x。
2. 找到 `dist/*.zip`，运行 `./scripts/package-installer.ps1 -Archive <zip>`，打印编译耗时。
3. `upload-artifact` 增加 `dist/*.exe`（`dist/*.sha256` 已覆盖安装包的 sidecar）。

`release.yml` 同样在 `release.ps1` 之后加这两步；发布时收集**全部** `dist/*.zip`、`dist/*.exe` 及全部 `.sha256` 作为 Release 资产，ZIP 的命名、校验和与发布顺序不变。

`release.ps1 -Publish` 保持只上传 ZIP：安装包只在 CI 的 Release 流程里产出，避免本地发布路径多一处需要单独验证的分支。

实测（本机 2026-11-06，7.1.1，增量 Rust 构建）：

- 已有 0.4.4 ZIP 时，`package-installer.ps1` 的校验+解压+`ISCC` 编译共约 25 秒，其中 `ISCC` 19.1 秒（`lzma2/max`，142 exe + 16 shim + 6 文档）。
- 从零构建 0.4.4 ZIP（Rust 已预热）并用 `release.ps1` 完整校验约 72 秒。
- 首次 CI 实测（run `42206804826`，commit `9f3a411`）：`Install supported Inno Setup` 6 秒——runner 上 PATH 里的 Chocolatey `ISCC.exe` 属于未注册版本，脚本按 `toolchain.json` 下载并静默安装 7.1.1，随后打印 `Inno Setup compiler: 7.1.1 at C:\Program Files\Inno Setup 7\ISCC.exe`；`Build EXE installer` 30 秒（`ISCC` 编译 25.2 秒）；整个 windows job 17 分 29 秒（主要耗时仍是 Rust 构建）。

## 7. 版本与签名

- 版本唯一来源：根 `Cargo.toml` 的 `workspace.metadata.ruxcmd.version`（当前 `0.4.4`）。
- 一致性要求：`manifest.json.suiteVersion` = workspace 版本 = 安装包 `ProductVersion` = 142 个 exe 的 `ProductVersion`；安装包与 exe 的 `FileVersion` 都是 `<major>.<minor>.<patch>.0`。
- 预发布版本照样出安装包：`AppVersion` 保存完整字符串（如 `0.5.0-beta.1`），`VersionInfoVersion` 只用数字段；升级/降级比较只看前三段数字，预发布与其同数字正式版视为同一版本（可互相覆盖）。
- 未签名的实际表现：
  - 仅当前用户 + 用户可写目录：无 UAC。
  - 所有用户：UAC 显示“未知发布者”。
  - 从浏览器下载的安装包带 MOTW，双击可能触发 SmartScreen“Windows 保护你的电脑”，需要“更多信息 → 仍要运行”；`gh release download` 等方式通常不带 MOTW，可能不触发。
  - Windows 11 开启 Smart App Control 或企业策略时，未签名安装包可能被直接阻止。
- 后续签名：Authenticode（signtool + RFC3161 时间戳）对 setup EXE 签名；证书（OV/EV）另议。

## 8. 风险与限制

- Inno 不是事务型安装器：中途失败可能留下半装状态，恢复方式是重跑安装包或卸载后重装。
- 升级时旧版已移除的 exe 会保留到最终卸载（见 §3.2）。
- PATH 写入是代码行为，若安装过程中被安全软件拦截，可能出现“文件已装但 PATH 未生效”；重跑安装包会补上（去重检查保证不重复）。
- `%`、`;`、`"` 的目录被拒绝是刻意收紧；极端合法路径（例如包含 `%` 的目录名）无法安装到自定义目录，可改用默认目录。
- 未签名带来 SmartScreen/UAC 提示；Smart App Control 可能直接阻止。
- 安装包内 exe 数量与版本资源的端到端校验依赖真实安装（需授权，见 §9）。
- 工具链限定 7.x：本机/CI 若只有 5/6/8 系，脚本会早期失败并打印实际版本与下载地址，不会退回到未验证的版本。

## 9. 本机验证方案（需用户授权后执行）

本机当前状态：PowerShell 7 可用；本机已装有 **Inno Setup 7.1.1**（`C:\Program Files\Inno Setup 7\ISCC.exe`），在支持区间内，无需再安装。以下步骤都会写注册表/PATH/文件，必须经用户逐项授权后再执行。

### 9.1 只读/可随时丢弃的检查（不需要安装）

```powershell
# 语法解析
pwsh -NoProfile -Command '$t=$null; $e=$null; [void][System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path scripts/package-installer.ps1).Path,[ref]$t,[ref]$e); $e.Count'

# 打印实际工具链版本（应为 7.x）
./scripts/packaging/inno/install-inno.ps1

# 需要先有 dist/ 下的 0.4.4 ZIP
pwsh -File scripts/package-installer.ps1 -Archive dist/ruxcmd-0.4.4-x86_64-pc-windows-msvc.zip
```

预期：解析 0 个错误；`install-inno.ps1` 输出 `Inno Setup 7.1.1 is already installed ...`；若工具链不属于 7.x，打包脚本在早期失败并打印实际版本与下载地址（不会开始解压或写 `dist/`）。

### 9.2 工具链现状（已满足）

本机 `ISCC.exe` 为 7.1.1（注册表 `Inno Setup 7_is1`），落在 `supportedMajorVersion=7` 区间内，`package-installer.ps1` 会打印 `Inno Setup compiler: 7.1.1 at C:\Program Files\Inno Setup 7\ISCC.exe` 后继续。需要复核时运行：

```powershell
./scripts/packaging/inno/install-inno.ps1
```

预期：输出 `Inno Setup 7.1.1 is already installed at C:\Program Files\Inno Setup 7\ISCC.exe`，不下载、不安装。若换到只有 5/6/8 系的机器，脚本会明确拒绝；需要时可用 `-Iscc` 指向合规路径，或让脚本按 `toolchain.json` 安装 7.1.1。

### 9.3 构建安装包（不安装到系统）

```powershell
pwsh -File scripts/package-installer.ps1 -Archive dist/ruxcmd-0.4.4-x86_64-pc-windows-msvc.zip
```

预期：`dist/ruxcmd-0.4.4-x86_64-pc-windows-msvc-setup.exe` 与 `.sha256` 生成；输出显示 `binaries: 142`、`shims: 16`、`docs: 6`、编译耗时。

### 9.4 首次安装（用户级 + 自选目录，授权项）

```powershell
$setup = "dist\ruxcmd-0.4.4-x86_64-pc-windows-msvc-setup.exe"
& $setup /CURRENTUSER /DIR="D:\Tools\RuxCmd\bin" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /LOG="$env:TEMP\ruxcmd-setup.log"
"exit=$LASTEXITCODE"
(Get-ChildItem "D:\Tools\RuxCmd\bin\*.exe").Count
(Get-ChildItem "D:\Tools\RuxCmd\bin\rux-*.cmd").Count
(Get-ItemProperty "HKCU:\Environment").Path
Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}_is1" | Select-Object DisplayVersion, DisplayIcon, InstallLocation
```

预期：

- `exit=0`；exe 数量 142；`rux-*.cmd` 16（不含 `rux-where`）。
- HKCU PATH 中 `D:\Tools\RuxCmd\bin` 恰好出现一次（末尾追加），其他项不变。
- 卸载键存在，`DisplayVersion=0.4.4`。
- 新开控制台执行 `rux-find --version` 能找到命令；`D:\Tools\RuxCmd\bin\find.exe --version` 正常。

### 9.5 同版本重装（授权项）

重复 9.4 的同一条命令（不带 `/DIR` 时应沿用旧目录、不弹目录页）。

预期：`exit=0`；目录不变；PATH 仍只有一条该目录；文件被覆盖（时间戳更新）。

### 9.6 跨版本升级（授权项）

先安装旧版安装包，再安装新版，各自记录 `DisplayVersion` 与目录。

预期：不弹模式/目录页；目录与旧版相同；`DisplayVersion` 变成新版本；PATH 仍只有一条；旧版本已移除命令的 `rux-*.cmd` 不残留。

当前仓库只有 0.4.4 的安装包，没有下一版本的安装包，因此本轮**无法真实执行升级**，也不允许为测试自造版本。替代方案：等下一个版本发布后用两份真实安装包执行本步骤；或在合并下一版本代码后用 `package-installer.ps1` 现场构建（需要该版本 ZIP 与已提交的 checkout）。

### 9.7 降级与改目录（授权项）

```powershell
# 先装 0.4.4，再运行 0.4.3 的安装包（当前没有 0.4.3 安装包，见下方说明）
& "dist\ruxcmd-0.4.3-...-setup.exe" /CURRENTUSER /VERYSILENT /SUPPRESSMSGBOXES
# 预期：非 0 退出，日志/消息提示已安装更高版本

# 已安装 0.4.4 后改目录
& $setup /CURRENTUSER /DIR="D:\Other" /VERYSILENT /SUPPRESSMSGBOXES
# 预期：非 0 退出，提示先卸载
```

**降级本轮无法真实验证**：需要“已安装更高版本 + 更高版本的安装包”这组前提，仓库里只有 0.4.4，`package-installer.ps1` 又要求 ZIP 版本与当前工作区一致，因此不能也不应为了测试构造 0.4.3 或 0.4.5 的安装包。替代方案：等真实的下一个版本发布后，先装新版、再运行本版安装包，验证非 0 退出与提示；`RuxCmd.iss` 的 `InitializeSetup` 已实现该比较逻辑。改目录拒绝用当前 0.4.4 安装包即可真实执行。

### 9.8 机器级安装（授权项，需要 UAC）

```powershell
& $setup /ALLUSERS /DIR="C:\Program Files\RuxCmd\bin" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
[Environment]::GetEnvironmentVariable('Path','Machine') -split ';' | Select-String 'RuxCmd'
```

预期：UAC 提示“未知发布者”；安装成功；`HKLM\...\Uninstall\{06BD...}_is1` 存在；机器 PATH 追加一条且只一条；HKCU PATH 未被修改。

### 9.9 卸载（授权项）

```powershell
& "D:\Tools\RuxCmd\bin\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
"exit=$LASTEXITCODE"
Test-Path "D:\Tools\RuxCmd\bin"
(Get-ItemProperty "HKCU:\Environment").Path -split ';' | Select-String 'RuxCmd'
Test-Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}_is1"
```

预期：`exit=0`；目录与 `rux-*.cmd`、`share` 被删除（可能留下空目录）；HKCU PATH 只剩用户原有项，`Search 'RuxCmd'` 无输出；卸载键不存在。

### 9.10 互斥检查（授权项）

```powershell
# 安装包已安装时
pwsh -File scripts/install.ps1 -Source <zip> -Force
# 预期：立即报错，提示先卸载 EXE 安装包；不写任何文件、不改 PATH

# 卸载后再跑
pwsh -File scripts/install.ps1 -Source <zip> -Force
# 预期：正常安装（安装包卸载后互斥检查应放行）
```

脚本版在用户 PATH 中时，运行安装包应被 `PrepareToInstall` 中止并给出卸载命令。

### 9.11 安装后 exe 版本资源（授权项）

```powershell
$dir = "D:\Tools\RuxCmd\bin"
Get-ChildItem "$dir\*.exe" | Where-Object Name -ne 'unins000.exe' |
  Where-Object { [Diagnostics.FileVersionInfo]::GetVersionInfo($_.FullName).ProductVersion.Trim() -ne '0.4.4' }
# 预期：无输出（142 个 exe 的 ProductVersion 全部为 0.4.4）
```

### 9.12 本轮验证记录（本机，2026-11-06）

产物：

- 本机构建（验证期间使用）：`ruxcmd-0.4.4-x86_64-pc-windows-msvc-setup.exe`，16,474,102 字节，SHA256 `c248b53860e3fc875a35c98b90348bb5a01523482c81d362c175ee02b4aa75fc`，`ISCC` 编译 19.1 秒。
- CI 构建（run `42206804826`）：同一文件名，16,442,094 字节，SHA256 `dd2ca321d6fd529023a3bf6f4c64553c96e38c79a0b85af706b6c80c102d8f1d`，sidecar 与实物一致。
- 两者版本资源均为 `0.4.4 / 0.4.4.0`；下表结果在两份产物上分别复现，CI 产物按 §9.1–§9.5 完整跑过一遍。

| 步骤 | 结果 |
| --- | --- |
| 首次安装（`/CURRENTUSER /DIR=%TEMP%\...`） | `exit=0`；142 exe、16 shim、6 文档；PATH 仅在末尾追加一条；`DisplayVersion=0.4.4`；HKCU 卸载键存在 |
| 同版本重装（首次尝试） | `exit=7`：Inno 7 的 `Inno Setup: App Path` 值名不匹配，报“已安装但目录无法读取” |
| 同版本重装（修复后） | `exit=0`；沿用原目录；卸载键仍为 1 个；PATH 不变 |
| 改目录重装 | `exit=7`，提示“不支持改目录”；目标目录未创建；PATH 不变 |
| 卸载 | `exit=0`；exe/shim/share/目录/卸载键全部清理；用户 PATH 与安装前快照**逐字符一致** |
| exe 版本资源 | 142 个 exe 的 `ProductVersion` 全部 `0.4.4`（0 个不一致），名称集合与 `manifest.json` 一致 |
| 互斥：安装包存在时跑 `install.ps1` | 立即报错并提示先卸载；未写文件、PATH 不变 |
| 互斥：脚本安装存在时跑安装包 | `exit=7`，`PrepareToInstall` 中止并给出 `install.ps1 -Uninstall` 命令；未创建目录、PATH 不变 |
| 升级到下一版本 | 未执行：仓库只有 0.4.4 安装包，且不允许自造版本 |
| 降级 | 未执行：需要“已装更高版本 + 更高版本安装包”，同样缺产物；不自造版本 |
| 所有用户（HKLM）模式 | 本轮未执行：需要 UAC 交互，且会改机器级 PATH/注册表 |

CI 产物（run `42206804826`，commit `9f3a411`，`ruxcmd-windows-x64-ci`）：ZIP、`ruxcmd-0.4.4-x86_64-pc-windows-msvc-setup.exe` 与两者的 `.sha256`；构建日志显示 `Inno Setup compile finished in 25.2s`、`binaries: 142`、`shims: 16`，安装包版本资源 `0.4.4 / 0.4.4.0`。

验证期间的所有安装目录均在 `%TEMP%` 下自建，结束后已卸载并删除；未触碰 `dist` 下的既有 ZIP/安装包。

## 10. 已落地的文件

| 文件 | 作用 |
| --- | --- |
| [scripts/packaging/inno/RuxCmd.iss](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/RuxCmd.iss) | Inno 安装/卸载脚本（固定 `AppId`、范围、目录、PATH、降级/改目录检查、卸载清理） |
| [scripts/packaging/inno/toolchain.json](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/toolchain.json) | 固定 Inno Setup 版本（7.1.1）、支持区间（7.x）与官方下载地址 |
| [scripts/packaging/inno/install-inno.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/packaging/inno/install-inno.ps1) | CI 与本机共用：探测、按需静默安装并复验 7.x |
| [scripts/package-installer.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/package-installer.ps1) | 从现有 ZIP 生成 `[Files]`、调用 ISCC 编译、校验版本资源并写 `.sha256` |
| [scripts/common.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/common.ps1) | 新增共享 shim 策略与 `Expand-ReleaseArchive` |
| [scripts/install.ps1](file:///C:/Users/Tim/Work/my-projects/RuxCmd/scripts/install.ps1) | 默认目录改为 `%LOCALAPPDATA%\RuxCmd\bin`、共享 shim、新增安装包互斥检查 |
| `.github/workflows/ci.yml` / `release.yml` | 支持区间内 Inno 安装、安装包构建、作为 Release 资产 |

注意：仓库提交时只包含上述源文件；`dist/` 下的既有产物与临时目录不纳入、不删除。
