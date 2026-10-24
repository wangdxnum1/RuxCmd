# P1 缺陷修复执行计划(Round 12)

**依据:** docs/command-review-2026-10-07.md 全部 P1 项(约 45 命令)+ Round 11 台账遗留清单。
**原则:** 沿用 Round 11 全局约束(先失败测试、--locked、一命令一 fix 提交、错误英文 stderr、不动 P2、禁止批量 allow)。文档同步收尾一次完成(Task F)。
**范围裁决:** 与 P0 修复重叠的 P1 不重做;"实现或移除"裁决时优先移除声明(成本低);大重写(sed/awk 完整解释器、find -exec)只做本轮列明的最小集。

## 批次(执行顺序)

| 批 | 命令 | 要点 |
|---|---|---|
| A1 | pkill, pgrep, kill | 去 tasklist/wmic/taskkill:复用 ps-sys(path 依赖)Toolhelp+OpenProcess+TerminateProcess;pgrep 无匹配 exit 1;pkill 校验终止结果;排除自身;kill:失败非零、-s 信号校验与 -l 全表、`kill -9 PID` 连写 |
| A2 | iostat, mpstat, sar | PDH(LogicalDisk/Processor 计数器)真实数据;iostat -d/-x 生效或移除;mpstat all/Average 行;sar -d 修或移除、CPU 真实、-c→保留但补 -u?最小:sar CPU 用 GetSystemTimes |
| B1 | time, top, uptime | time:GetProcessTimes 填真实 user/sys、GNU 格式 `0m0.003s`、-p;top:GetSystemTimes CPU%、uptime:默认 procps 格式(up 时长+用户数,负载省略声明) |
| B2 | vmstat, free, df | vmstat r 列语义修正(id 数据源顺带);free:-t/-s/-w、available 列;df:GetVolumeInformationW 真实类型+-T、失败报错退出 1、Use% 向上取整 |
| C1 | w, who | WTSEnumerateSessions/WTSQuerySessionInformation 会话列;boot time 用 GetTickCount64(去 systeminfo);w 三列头部尽力 |
| C2 | ps(三包) | pcpu 由 start_time+GetProcessTimes 计算真实 CPU%;stat 列去死代码(或真实状态);token.rs 句柄泄漏修复;-a/-x 语义;-C 精确匹配;--headers-repeat 移除;浮点/负值排序;stime 本地时区 |
| D1 | grep, cut | 非UTF-8 按字节处理(不静默截断);grep 单文件无前缀、补 -E/-F/-w/-l/-h/-H;cut -b 字节透传 |
| D2 | sort, head, tail | sort:非UTF-8、失败退出码、`-` stdin;head 行模式字节透明(CRLF/无尾换行/二进制);tail:-c -f、-n +N、行模式字节透明 |
| E1 | sed | 最小命令集:p/d/y/s 保留正则?——保持子串匹配已声明,新增:地址(行号+N$,正则子串地址)、p/d 命令、`sed 's/a/b/' file` 位置参数、-i 临时文件原子替换、CRLF 保留、非UTF-8 按字节 |
| E2 | awk | if 条件求值修复(Debug 串 bug)、字段残留清理、print 多参数、else/for 执行、next 语义、NR 跨文件、非数值模式真值 |
| F1 | paste, join, fold, unexpand, fmt, tr | paste \r 剥离+-d 循环;join 输出形状(键+剩余字段)、默认空白分隔;fold 非UTF-8/-b 字节;unexpand ≥2 空白;fmt -c 无值形态+下溢+-s 断词;tr 转义序列+字符类+SET2 挤压 |
| F2 | mv, rm, ln, cp, open | mv:拒绝目录覆盖非目录、跨卷 copy+delete、panic 修复;rm -f 失败报错退出码;ln 按用户书写存目标、-v 改 verbose;cp 默认覆盖(GNU)+no-clobber 语义修正(声明);open:ShellExecuteW 替代 cmd/powershell(修 -R/注入/& 截断) |
| G1 | curl, wget, ping | curl:默认只输出 body、-d 隐含 POST、-L 可关、-f、-I、-s 语义;wget:URL 尾 / 文件名、? 截断解码、超时、退出码 8;ping:丢包退出码 1、状态消息区分 |
| G2 | netstat, xargs, find | netstat:-a/-n 生效、-o PID(GetExtendedTcpTable);xargs:-p 从 CONIN 读、命令行长度分批、-0;find:maxdepth 偏移修复、-type 未知值报错、错误退出码 |
| H1 | chmod, test, env, yes, true, false, sleep, pwd, hostname | chmod 符号模式 u/g/o/a+-rwx;test -eq 等数值+单参形式+-z/-n/!;env NAME=VAL+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错+GetComputerNameExW |
| H2 | md5sum, sha256sum, od, iconv, numfmt | md5/sha256:* 标记、BOM、失败继续汇总退出码、--status;od:-t x1 尺寸、-t f 十进制浮点、数值模式不加 ASCII 列;iconv:-f 严格(去 BOM 嗅探)、--list 补全;numfmt:-d 输出分隔符、from=auto 基数 |
| I1 | tar, unzip, gunzip, csplit, patch, basename | tar:-C/-O/成员名选择;unzip:流式解压(去 read_to_end 内存放大);gunzip:-c;csplit:降序行号报错;patch:hunk 按计数解析、多文件补丁、.rej 内容正确;basename 双操作数 NAME SUFFIX |
| F | 收尾 | compatibility 双语全量同步、CHANGELOG、build.ps1 Verify、release 冒烟、台账 |

## 验收

- 上述命令的 P1 项全部:已修复(附测试)/ 已实现或移除声明 / compatibility 声明为差异;
- Verify 与 release 冒烟全绿;台账 Round 12 节回填。
