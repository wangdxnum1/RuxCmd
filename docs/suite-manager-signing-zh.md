# RuxCmd Suite Manager 代码签名方案

**简体中文** | [English](suite-manager-signing.md)

> 状态：**未采纳，仅作未来参考**（2026-11-08 维护者决定暂不签名）。撰写日期 2026-11-08。
> 本文保留完整的调研结论与备选对比，供日后决定启用签名时直接复用；**当前不申请、不接入 CI、不作为任何阶段的前置条件**。
> 适用对象：Suite Manager 的 Tauri 安装包 / 可执行程序；顺带覆盖现有 RuxCmd 发布资产（setup.exe 与 142 个 exe）。
> 依据：[设计说明](suite-manager-design-zh.md) 第 7 节「安全与信任链」。

## 0. 先纠正一个常见误解

**签名不能立刻消除 SmartScreen 首次运行警告。**

微软 [SmartScreen 文档](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/) 明确：它是**基于声誉**的判定——检查「文件 / 数字签名证书 / URL」是否已有已建立的声誉；没有声誉就提示警告。因此：

| 手段 | 效果 |
| --- | --- |
| 不签名 | 一定警告，且用户看到「未知发布者」 |
| OV 标准证书 / 免费 OSS 证书 | 显示正确发布者，但初期仍可能有 SmartScreen 提示；随下载量与时间累积声誉后消失 |
| EV 证书 | 通常能更快建立声誉（历史上有即时声誉待遇），但成本与资格门槛高 |
| 向微软提交文件复核 | 误报时的补救通道，[提交入口](https://www.microsoft.com/wdsi/filesubmission/) |

所以签名的真实收益是：**可验证的发布者身份 + 完整性 + 时间戳 + 声誉积累的起点**，以及让 winget / 企业环境 / 用户愿意安装。设计文档里「UI 自己未签名会断信任链」的判断成立，但方案不能承诺「签完就没警告」。

## 1. 推荐方案：SignPath Foundation（OSS 免费）+ SignPath.io CI 集成

### 1.1 为什么是它

RuxCmd 的条件与 SignPath 高度匹配：

- MIT 许可（OSI 批准）、公开 GitHub 仓库、已持续发布（v0.5.11，当天 11 个版本）→ 满足 [SignPath Foundation 条款](https://signpath.org/terms) 的 OSS License / Released / Maintained / Documented 要求。
- 项目**没有专有组件**、不含 hacking tools、不收集用户数据、安装器提供卸载 → 满足条款其余约束。
- **免费**（[signpath.org](https://signpath.org)："For OSS projects, our services are free of charge"），无证书采购/续期成本。
- 私钥由 SignPath 生成并存放在 **HSM**，不落到开发者机器或 CI runner——正好解决「云构建里没法插 USB token」这个经典难题。
- 签名会**验证二进制确实由仓库源码自动构建**而来，与项目「发布溯源 / 逐文件校验 / 门禁」的方法论天然一致：把「manifest 的 sourceCommit」升级成「密码学可验证的来源」。

### 1.2 代价与约束（必须接受）

| 约束 | 对 RuxCmd 的影响 |
| --- | --- |
| 每次发布需**人工审批签名请求** | 与现有「Release workflow 手动触发」一致，不破坏流程；但 automation 闭环会多一个人工点 |
| 证书签发主体是 **SignPath Foundation**，不是 Tim / RuxCmd | 用户看到的发布者是 SignPath Foundation；项目主页需按条款写明 Code signing policy |
| 必须在项目主页与下载/发布页使用 **"Code signing policy"** 字样，并列出：免费签名提供方、团队角色（Authors / Reviewers / Approvers）、隐私声明 | 需新增一节文档 + 发布页文案；一次性工作 |
| 所有签名二进制的**产品名/产品版本元数据必须设置并强制** | RuxCmd 已有版本资源基建（`ruxcmd-build` 生成 exe 版本资源），Tauri 侧需在 `tauri.conf.json` / Windows 资源里补齐 |
| 团队全员 MFA（SignPath 与 GitHub） | 单人维护者：开启 GitHub MFA 即可 |
| 签名范围限于**自己源码构建的产物** | 只签 Suite Manager 与 RuxCmd 自身 exe；不签第三方依赖二进制（本项目无此情况） |
| 条款为 Draft，SignPath 保留随时暂停/撤销权利 | 属于外部依赖风险，见第 4 节备选 |

### 1.3 落地步骤

1. **申请**：在 [signpath.org/apply](https://signpath.org/apply) 提交，指向 `github.com/wangdxnum1/RuxCmd`；说明项目已发布、MIT、无专有组件。
2. **写 Code signing policy**：新增 `docs/code-signing-policy-zh.md` / `code-signing-policy.md`，并在 README 与 Release 说明里用 "Code signing policy" 字样链接。内容包含：
   - "Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org)"
   - 团队角色（单人维护者：Authors / Reviewers / Approvers 均为仓库 owner，可引用 GitHub 权限组）
   - 隐私声明：本项目**不向未由用户明确请求的网络系统传输任何信息**（注意：Suite Manager 的「检查更新」会请求 `api.github.com`，需在政策里如实列出，不能套用模板里的"不传输"措辞）
3. **配置 SignPath.io**：创建 project、artifact configuration（强制 product name / version 元数据）、signing policy；连接 GitHub Actions，签名请求由 CI 构建产物发起。
4. **CI 集成**：在 Release workflow 增加签名 job——构建产物上传 SignPath → 人工审批 → 下载已签名产物 → 再生成 `.sha256` / 发布。
   - **顺序很关键**：必须**先签名再算哈希**。现有流程是「打包 → 生成 `.sha256` → 上传」，若插入签名步骤，ZIP / setup.exe 的字节会变，哈希与 `check-manifests` 清单都要在签名后重算。
   - 建议把签名作为独立可选 job，先只签 **Suite Manager 安装包**，不动已验证的 142 exe 打包链；等跑通再扩展到 setup.exe。
5. **时间戳**：使用 RFC 3161 时间戳服务，保证证书过期后签名仍有效。SignPath.io 支持外部时间戳，配置时显式启用。
6. **验证**：发布后按项目既有收尾门禁核对——`Get-AuthenticodeSignature` 检查状态 Valid、签名者、时间戳；`Get-FileHash` 与 `.sha256` / GitHub asset digest 三方一致。

## 2. 备选方案对比

| 方案 | 成本 | 私钥管理 | 资格门槛 | 适用判断 |
| --- | --- | --- | --- | --- |
| **SignPath Foundation（推荐）** | 免费 | 对方 HSM，零接触 | OSS / 已发布 / 无专有组件 / 人工审批 | **首选**，与项目条件最匹配 |
| Azure Artifact Signing（原 Trusted Signing） | 按量付费，Basic / Premium SKU；[定价页](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) 需报价 | Azure 托管，FIPS 140-3 Level 3 HSM | 需 Azure 订阅 + 身份核验；个人身份核验较麻烦 | 若 SignPath 申请被拒、或未来需要企业级 SLA / 私有信任场景 |
| 自购 OV 代码签名证书 | 年费（数百美元级，各 CA 不同） | 2023 起 CA 强制 HSM/token 存储，CI 集成仍需云签名服务 | 个人可买，需身份验证 | 想让发布者显示为自己名字时；但每年续费 + 管理负担 |
| 自购 EV 代码签名证书 | 年费更高 | 硬件 token | 需 organization 身份核验，个人难申请 | 追求最快 SmartScreen 声誉；对个人维护者性价比低 |
| 暂不签名 + 文档声明 | 0 | — | — | **当前采用**（2026-11-08 决定）；winget 清单已声明未签名。代价是首次运行 SmartScreen 警告与「未知发布者」，Suite Manager 会放大这个短板——以 UI 内如实告知 + 三方哈希校验缓解 |

## 3. 分阶段建议（当前不执行）

> 下列步骤**仅为日后启用签名时的实施顺序参考**，当前不启动任何一步。P0–P4 的 Suite Manager 开发不因签名阻塞。

| 阶段 | 动作 | 风险 |
| --- | --- | --- |
| 启用时（先决） | 提交 SignPath 申请、起草 Code signing policy 文档、确认发布者身份可接受 | 低 |
| 启用时（CI） | 落地 CI 签名 job，先只签 Suite Manager 安装包；同步把「先签名后算哈希」写进发布流程 | 中，会改动 Release workflow |
| 启用时（扩展） | 扩展签名到 RuxCmd `setup.exe`；更新 winget / scoop 清单移除「未签名」声明；核对哈希链 | 中 |
| 启用后 | 若下载量起来仍有 SmartScreen 提示，走微软文件提交复核；或评估升级 EV | 低 |

**若日后启用**：签名一旦插入发布流程，必须「先签名 → 重算 `.sha256` → 回填清单 → `check-manifests`」，因为签名会改变 ZIP / setup.exe 字节；这会改动 Release workflow，需单独成轮验证。

## 4. 风险与对策

| 风险 | 对策 |
| --- | --- |
| SignPath 申请被拒（条款含 "verifiable reputation" 的软性判断） | 备选 Azure Artifact Signing；或先用自购 OV。申请时强调已发布版本数、CI 可复现构建、逐文件 SHA256 与 manifest 溯源 |
| 证书主体是 SignPath Foundation，用户疑惑 | 在 About 页「发布者」字段与 Code signing policy 里明确说明 |
| 人工审批拖慢高频发布 | 项目本来就是手动触发 Release；若日后自动化，可评估 Azure 方案 |
| 签名改变 ZIP/setup 字节，哈希链断裂 | 严格执行「先签名 → 重算 `.sha256` → 回填清单 → `check-manifests`」；把这一步纳入 roadmap 4.2 的 post-release sync job |
| 条款为 Draft、服务可能变更 | 签名方案可替换：CI 里把签名 job 做成可插拔；哈希与校验链不依赖具体签名提供方 |
| SmartScreen 仍提示 | 如实向用户说明「签名 ≠ 无警告」；提供校验方法；必要时提交微软复核 |

## 5. 验收清单

- [ ] `Get-AuthenticodeSignature <setup.exe>` 状态为 Valid，含 RFC 3161 时间戳
- [ ] 签名后的 ZIP / setup.exe 的 `.sha256` 与 GitHub asset digest、check-manifests 清单三方一致
- [ ] 项目主页与每个 Release 页含 "Code signing policy" 链接
- [ ] About 页显示发布者与签名状态
- [ ] winget / scoop 清单移除「未签名」声明（如适用）
- [ ] 文档明确「签名不保证立即免除 SmartScreen 警告」

---

*参考：[SignPath Foundation](https://signpath.org/) · [SignPath 条款](https://signpath.org/terms) · [Azure Artifact Signing 概览](https://learn.microsoft.com/en-us/azure/artifact-signing/overview) · [定价](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) · [Microsoft Defender SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/)*
