# Round 15 设计与执行计划:高频缺口命令移植

**日期**:2026-10-25
**依据**:2026-10-25 迭代方向评估(方向三"命令族完整性");审查质量基线(Round 11-14 后 P0/P1 清零、差分测试基线 274 过/0 失败)。

## 1. 目标

一次性补齐 GNU coreutils 高频缺口,命令数 111 → 125;新命令全部按现行标准交付:clap 解析、行为测试、compatibility 双语条目、difftest 纳入(有 GNU 参照者)、冒烟策略。

## 2. 新命令清单与适配设计

| 命令 | 功能 | Windows 适配 |
|---|---|---|
| rmdir | 删空目录 | `fs::remove_dir`;`-p` 逐级上溯;`--ignore-fail-on-non-empty` |
| link / unlink | 硬链接创建/删除 | `fs::hard_link` / `fs::remove_file`(复用 ln 经验:目标按书写形式) |
| mktemp | 安全临时文件/目录 | 自实现:模板尾部 `XXXXXX` 随机替换,`O_EXCL` 语义(create_new),`-d` 目录、`-p DIR` 前缀、`-t`;不引入 tempfile 运行时依赖 |
| install | 复制并设置属性 | `fs::copy` + `-d` 建目录 + `-m` 映射 readonly(复用 chmod 语义);`-D` 建目标父目录 |
| sha1sum | SHA-1 校验 | sha1 crate;复制 sha256sum 结构(含 check/--tag/--status/两空格) |
| sha512sum | SHA-512 校验 | sha2 crate(Sha512);同上 |
| cksum | POSIX CRC32 校验 | 手写 CRC-32(多项式 0x04C11DB7,POSIX 初始化/终值规则),无新依赖 |
| logname | 当前登录名 | GetUserNameW,失败回退 USERNAME 并声明 |
| users | 登录用户单行列表 | 复用 w/who 的 WTSEnumerateSessionsW 小函数(允许 crate 内复制) |
| nohup | 断连保护运行 | spawn 时 CREATE_BREAKAWAY_FROM_JOB + CREATE_NEW_PROCESS_GROUP;stdout 为终端时重定向 nohup.out;等待并透传退出码 |
| watch | 周期执行并全屏显示 | ANSI 清屏循环;`-n`(默认 2s)、`--no-title`;`-d` 高亮声明不支持;Ctrl+C 退出 |
| traceroute | ICMP 路由跟踪 | 复用 ping 的 IcmpSendEcho,TTL 递增 1..30,每跳 3 探测;TTL 超期→路由器地址,超时→`*`,到达目标→结束 |
| vdir | ls 的长格式默认别名 | ls crate 增加 lib target 暴露 `run(args)`,vdir 薄壳注入 `-l` |
| ls -h 短选项 | (补齐)现有 --human-readable 加短名 | ls/src/cli.rs 一行 |

## 3. 约束

- 沿用全局约束:先测试后实现、`--locked`、错误英文 stderr、一逻辑单元一提交、禁止批量 allow;
- 不引入新运行时依赖(sha1/sha2 为既有生态标准 crate,首次使用需在台账登记理由);
- watch/traceroute/mktemp 无逐字节 GNU 参照(或参照缺失),以行为测试 + 冒烟代替 difftest;
- rmdir/link/unlink/mktemp/install/cksum/sha1sum/sha512sum/users/logname/vdir 纳入 difftest 矩阵(缺失 GNU 参照自动跳过)。

## 4. 执行波次

| 波 | 内容 |
|---|---|
| 1 | rmdir + link + unlink + mktemp(代理 A);sha1sum + sha512sum + cksum(代理 B) |
| 2 | install + logname + users(代理 C);nohup(代理 D) |
| 3 | watch(代理 E);traceroute(代理 F) |
| 4 | ls -h + vdir lib 化(代理 G) |
| F | difftest 矩阵扩展、compatibility 双语、CHANGELOG、Verify、台账、推送 |

## 5. 验收

- 14 个新 crate 全部 Verify 通过;有 GNU 参照者纳入 difftest 且无失败;
- compatibility 双语 + CHANGELOG + 台账更新;
- 全部推送。

## 6. 接续执行裁决（2026-10-25）

- 用户确认：本轮完成 14 个新命令的集成验收，`-z` 系列单独下一轮；不增加 `tracert` 别名或 winget 发布任务。
- 既有四波实现已在 4c9a1a5、afffc6a、87bb6a3、a5076db 落地；本次沿用当前 checkout 并切换到 `codex/round15-completion` 保留未提交差分脚本，避免重复开发。
- watch 实际已实现整行 `-d` 高亮及 `-g/-e`；文档按实际行为声明。mktemp 的随机输出仅规范化随机名后比较，同时检查文件系统快照，不再整体豁免。
- 安全修正：mktemp 复用已锁定的 windows-sys/BCryptGenRandom，替换可预测的时钟种子 PRNG；install 复用 windows-sys 获取文件身份，替换目标条目而非改写其链接指向的文件。Cargo.lock 仅登记这两个既有依赖引用；sha1 为 SHA-1 实现所需，sha2 沿用既有生态。
- nohup：默认尝试脱离控制台及 Job；实际测试环境即使在沙箱外也有禁止 breakaway 的 Job。保留 DETACHED_PROCESS 重试，并明确警告 Job 生命周期限制；不宣称任意 Job 下均可存活。完全拒绝运行会使常见的受 Job 管理终端无法使用此命令；取舍是这类环境下作业关闭仍可能终止子进程。
- 硬链接、ICMP 与脱离进程测试在普通 Windows 权限下验收；沙箱拒绝 API 的失败不作为通过或代码缺陷。
- 本地验收已通过：853 Rust tests、125 smoke、8 behavior、35 脚本测试；GNU release 差分 362 pass / 0 fail / 31 skip（seed 20261025，variants 2）。本地 ZIP 经 DLL/校验和/解压验证，未发布；详见 `.superpowers/sdd/progress.md` Round 15 记录。
