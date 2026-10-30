# Round 18 缺失命令补齐设计(9 个新命令)

日期:2026-10-29。基线:main(Round 17 合并后)。

## 目标

按 Round 15 打法补齐 GNU coreutils 中常用且 Windows 可落地的缺失命令,125 → 134:

| 命令 | 范围 | 参照来源(本机 Git Bash coreutils 8.32 实测) |
| --- | --- | --- |
| nproc | 可用处理器数;--all、--ignore=N(下限 1) | `nproc`→20,`--ignore=2`→18 |
| printenv | 全量/按名查询;-0 NUL 分隔;任一缺失 exit 1 | FOO=bar 实测 |
| expr | 整数四则/比较/\|&、BRE 锚定匹配(:、match)、substr/index/length、括号;退出码 0/1/2 | `1 + 2`→3、`0`→exit 1、`abc : 'a\(.*\)'`→bc |
| truncate | -s/--size([+-] 前缀、K/M/G/T 二进制与 KB/MB 十进制后缀)、-c 不创建;-r 记录为差异 | `-s 100` 实测 |
| base32 | RFC 4648 编解码;-d、-w(默认 76,0 不换行)、-i 忽略非法字节 | `hello`→NBSWY3DPBI====== |
| sum | BSD(默认/-r,1K 块,旋转校验和,无文件名)与 SysV(-s,512 块,含文件名) | `36979     1`、`542 1 s.dat` |
| tsort | 拓扑排序;奇数 token/环报错退出 1 | `a b`→a b |
| pathchk | 路径合法性子集:空段、组件 >255、总长 >4096;-p 便携字符集 | 合法路径 exit 0 |
| hostid | 机器名 FNV-1a 32 位散列,8 位小写十六进制 | 格式 `75306b33` |

明确不做(记录遗留):chgrp(需复用 chown 组逻辑,单独一轮)、dd(大)、b2sum(需引入 BLAKE2 依赖)。

## 约定

- TDD;--locked;一命令一提交;错误英文 stderr;不改动现有命令;冒烟清单与双语兼容性表同步。
- expr 引入 regex 依赖仅用于 BRE 转换;其余零新增依赖(nproc 复用 windows-sys)。
- 所有新命令单文件 main.rs(内联 clap Args),沿 0.3.x 后惯例。
