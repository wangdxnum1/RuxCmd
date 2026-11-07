# Changelog

## Unreleased

- bc:支持 `scale=N`（0..=64）小数精度——scale>0 时 `/`、`%`、`^` 负指数、`sqrt()` 产生小数（i128 定点，约 38 位有效数字）；十进制字面量；非任意精度（超出报 overflow）。
- bc:支持 `ibase`/`obase` 进制转换（2..=16，ibase ≥ 11 时 A-F 为数字位；obase ≠ 10 时整数按 obase 输出）。
- cut:新增 `-n`（不拆分多字节 UTF-8 字符）与 `--complement`（反转选择集）。
- head:`-n` 与 `-c` 后出现的选项生效（移除互斥报错，对齐 GNU）。

## 0.5.0 — 文件批处理联合迭代与发布基线稳定(2026-11-07)

- 文件批处理联合迭代：xargs 改为 UTF-8 流式解析，支持引号/转义、-I/-r、完整标记替换、NUL 空参数与明确子状态处理；子命令参数不再被误作 xargs 选项，未启用替换时花括号保持字面值。
- xargs 与 find -exec + 共用含引用开销的 8000 UTF-16 单位预算；-n 不绕过长度限制，find 每个动作独立分批并继续汇总失败。满批即执行可能影响后续遍历；非法 UTF-8 和超长项明确报错。
- 新增真实 find→xargs→cp 字节管道、find -exec + 与逐文件 SHA256 场景，纳入 Verify 和 ZIP 解压验证；测试工具 stdin 显式使用 UTF-8，避免本机代码页破坏中文校验清单。详见 docs/file-batch-usage.md。
- 稳定发布基线：兼容表校验纳入 `build.ps1 -Task Verify`，检查本次构建的 Debug/Release 目录并在失败时停止；CI、Release 与本地发布共用该门禁。
- Release 与 CI 使用相同的依赖缓存策略，不缓存工作区命令产物；补齐 cut/diff/mktemp 选项清单及中文 diff 说明。
- 新增按版本号与提交定位的迭代状态页，保留历史轮次记录。

## 0.4.9 — sed 反向引用、diff 换行标记(2026-11-07)

- sed:BRE 与 ERE 模式的反向引用 `\1`-`\9` 支持（经 ruxcmd-core 翻译器 fancy-regex 路径，地址与 s 命令均适用）；无效反向引用（前向/越界/引用未闭合分组）启动报错 Invalid back reference 退出 2；非 UTF-8 数据按原始字节匹配不变。
- diff:字节级读行改造（read_until 保留行尾信息）；末行无换行时按 GNU 输出 \ No newline at end of file 标记（普通与 -u 均适用）；缺尾换行参与行相等比较（-b 例外折叠）；支持 `-` 作操作数读 stdin；`-r/--recursive` 声明但使用报错退出 2；-u 相同输入无输出。
- expr:BRE 反向引用已升级（fancy-regex 回溯引擎，嵌套分组编号正确）。

## 0.4.8 — bc scale、expr 反向引用、cut --complement、mktemp dry-run(2026-11-07)

- bc:支持 `scale=N`(0..=64)赋值——scale>0 时 `/` 输出按 scale 截断的小数(i128 定点长除法,~38 位有效数字)、`%` = a−(a/b)·b、`^` 负指数与 `sqrt()` 产生小数;支持十进制字面量(1.5、.5);非零值打印完整小数位(前导 0)。仍非任意精度:超出报 overflow。
- expr:BRE 反向引用 `\1`-`\9` 支持(fancy-regex 0.20,纯 Rust 回溯引擎);POSIX 分组 `\(...\)` 编号保持嵌套语义;非法反向引用(前向/越界/未闭合)报 Invalid back reference 退出 2,与 GNU 8.32 一致。
- cut:新增 `--complement` 反转选择集(按行序输出)。
- mktemp:新增 `-u/--dry-run`(仅打印路径不创建)与无值 `--tmpdir`(用 $TMPDIR,否则 %TEMP%)。

## 0.4.7 — column fill、grep fnmatch、dd conv 扩展(2026-11-07)

- column:默认 fill 模式与 util-linux 2.40.2 逐字节一致(100 组固定输入 × 5 种宽度全 MATCH)——整行为词条、空行忽略、TAB 制表位对齐、列优先填充，词条宽 ≥ 终端宽时逐行原样输出；宽度仅读 COLUMNS 环境变量（默认 80）；-t 表格模式行为不变。
- grep:`--include`/`--exclude` glob 支持 fnmatch 字符类（[abc]、[a-z]、[!abc]、[]x]、转义），翻译规则与 GNU grep 3.0 实测一致。
- dd:`conv=sync`（短读补 NUL 到 ibs，修正 records in 计数时机）、`conv=nocreat`、`conv=fullblock`/`iflag=fullblock`（读侧短读合并）；conv/iflag 非法值错误格式对齐 GNU。

## 0.4.6 — 脚本必备 GNU 对齐(2026-11-07)

- touch:`-r/--reference`(取参考文件时间)、`-t STAMP`([[CC]YY]MMDDhhmm[.ss],本地时间经 TzSpecificLocalTimeToSystemTime 转换,DST 正确)、`-d/--date` 子集(@epoch、YYYY-MM-DD[THH:MM[:SS]]);源优先级 -r>-d/-t>now,-t 与 -d/-r 组合报 GNU 同款错误(8.32 实测对齐)。
- head:`-n -K`/`-c -K` 负计数(输出除末尾 K 个记录/字节,GNU 语义,-0 表示全部);非法数字按 GNU 措辞退出 1。
- comm:接受 `-` 作为任一操作数(两个 `-` 报错退出 1);`--total` 总计行(计数为未抑制的完整计数,-z 模式行尾 NUL);-1/-2/-3 的 TAB 前缀压缩 8 组旗标组合与 GNU 逐字节一致。
- du:`--max-depth=N`(GNU 数值解析含 0x/八进制)、`-a/--all`(文件与目录同深度计)、`-x/--one-file-system`(按卷剪枝)、`--exclude=PATTERN`(可重复,组件级 glob);`-a×-s` 报错退出 1、`-s×--max-depth` warning 分支与 GNU 一致;输出路径分隔符统一为 `/`。34 个 apparent-size 场景与本机 GNU 8.32 逐一比对全 MATCH。
- cp:`-a/--archive`(= -pR)、`-u/--update`(源 mtime 严格更新或目标缺失才复制,相等跳过且先于 -n/-i/-f)、`-t DIR`、`-T`;四类 GNU 错误文案复刻(exit 1 + Try 行);-t 与 -T 互斥。
- find:布尔运算符 `-o/-or`、`-a/-and`、`!/-not` 与 `\( \)` 分组(! > -a > -o,顺序短路求值,隐式 -print 补全);`-newer FILE`、`-newermt/-newerat DATE`(日期子集,空串=今日 00:00);表达式语法错误措辞复刻 GNU;55 个 find 测试全绿。
- diff:普通格式按 GNU 合并相邻删除+添加为单条 c 命令并折叠区间(0a…/…d0 锚点特例),与 GNU diffutils 3.12 逐字节对照一致(1800 组随机模糊验证:退出码 100% 一致、补丁应用 100% 还原);重复行等长 LCS 对齐选择差异仍存(输出仍为最短 diff)。
- 修同步的旧行为锁定测试:seq 尾换行、cut 无分隔符透传、nl 终止末行、sed 保留缺失尾换行、md5sum/sha256sum 两空格、wc stdin 裸计数(verify-behavior.ps1 同步)。

## 0.4.5 — 发布溯源修正、安装包纳入源码树(2026-11-06)

- 修正 0.4.4 的发布溯源缺口：`v0.4.4` tag 指向的提交早于安装包工具链，Release 中的 ZIP 与 setup 来自不同构建。本版的 ZIP 与 setup 由同一个 tag 提交、同一次 CI 构建产出，`manifest.json` 的 `sourceCommit` 与 tag 提交一致且 `dirty=false`。
- 安装包纳入正式源码树：新增 `scripts/packaging/inno/`（`RuxCmd.iss`、`install-inno.ps1`、`toolchain.json`，固定 Inno Setup 7.1.1，接受 7.x、拒绝 5/6/8）与 `scripts/package-installer.ps1`；CI 与 Release 均构建并上传 `ruxcmd-<version>-x86_64-pc-windows-msvc-setup.exe` 及其 `.sha256`。
- 安装包行为：首次安装可选“仅当前用户 / 所有用户”并自选目录（默认 `%LOCALAPPDATA%\RuxCmd\bin` 或 `%ProgramFiles%\RuxCmd\bin`）；升级与同版本重装就地覆盖并沿用原目录；降级拒绝；卸载移除安装文件、`rux-*.cmd`、`share`、PATH 条目与卸载项。
- PATH 与 `rux-` 前缀：安装结束把安装目录追加到用户级或机器级 PATH（卸载时精确移除该条目）；`C:\Windows\System32` 与 PowerShell 别名会抢先裸名，因此冲突名请使用 `rux-find`、`rux-cat`、`rux-ls` 等 shim（本套件实际生成 16 个），非冲突名可直接使用。PATH 变更只对新开进程生效，旧终端或 IDE 需重开。
- 安装包未签名，首次运行可能出现 SmartScreen 提示；可用 `Get-FileHash` 对照同名 `.sha256` 校验 ZIP 与 setup。
- 命令行为修正（0.4.4 tag 之后合入）：grep `-c` 对齐 GNU（按输入逐个输出 `name:count`，含 0 计数、无总计，并遵循 `-h`/`-l`/`-L`/`-m 0`）；du 后序遍历全部子目录、总是追加 `-c` 总计并按目录身份去重；find 按命令行顺序求值（隐式 `-and` 短路），`-delete` 可与 `-print`/`-print0`/`-exec` 组合。
- 工程：CI 改用 Swatinem/rust-cache（key 覆盖 `Cargo.lock`、`rust-toolchain.toml`、`.cargo/config.toml` 与该 workflow），工作区 crate 产物不缓存，保证 142 个 release 二进制由当前运行产出。
- 文档记录 Inno 7 的卸载值名变更（`Inno Setup: App Path`）、许可横幅、16 个 shim 的真实数量，以及 CI 与本机安装/重装/改目录拒绝/卸载/PATH 精确恢复的验证结果。

## 0.4.4 — 语义收口、安装与打包基线、awk 流式化(2026-11-06)

- du：默认改为 GNU 的 1K 已分配块（GetCompressedFileSize），新增 `-b/--bytes`、`-c/--total`、`-h` 与 `--apparent-size`；硬链接每次调用只计一次。已知差异：MSYS GNU du 按 4K 簇估算，在分配粒度等于逻辑大小的文件系统上块数会不同（apparent 模式一致）。
- cp：默认覆盖已存在的可写目标对齐 GNU；`-n`/`-i` 以最后出现者为准、`-n` 恒压过 `-f`，覆盖只读目标必须带 `-f`，`-i` 提示写 stderr，递归单条失败不中断其余条目但整体退出 1；`-p` 保留只读属性与访问/修改时间（不可映射的 POSIX mode 位与 owner/group 除外）。
- mv：覆盖、不覆盖与 `-f` 的优先级对齐 GNU；覆盖只读目标先清属性，删除失败时回退。
- uptime `-p` 采用 GNU 的英文措辞；zip 从 PowerShell shim 改为 zip crate 的原生实现。
- find 新增 `-print0`、`-delete`（隐式 -depth）、`-size`、`-mtime`、`-type l`、`-exec`（`{} \;` 与 `{} +`）；未知谓词与非法 glob 报错退出 1。grep 新增 `--include`/`--exclude`/`--exclude-dir`、`-m/--max-count`、`-z/--null-data`、`-L/--files-without-match`，并修复第 max-count 条匹配后仍输出尾部上下文。
- 新增 `scripts/install.ps1`：先按 SHA256 校验归档，安装到用户目录，生成 `rux-` 前缀 shim，只修改用户 PATH；支持 `-Uninstall`/`-RemoveInstallDir`。新增 scoop 与 winget 清单源（`scripts/packaging/`）。
- 文档与兼容表收敛为单一事实源：命令/包计数、wc stdin 说明、各命令验证状态、awk split 注释等统一；新增 `scripts/compat-check.ps1`，在 CI 中校验兼容表列结构、双语行数、选项与 `--help` 的一致性，以及 README/开发文档/兼容表/脚本 README 引用的计数。
- 工程基线：clippy 以 `-D warnings` 全仓通过并纳入 CI；发布 EXE 数（142）与 `cargo metadata` 的 bin 目标逐一核对。
- awk 内存有界：主输入与 `getline < file` 由一次性预加载改为按记录流式读取，峰值工作集不随输入增长（266 MB 输入实测：主输入 372 MB → 7.4 MB，`getline < file` 626 MB → 7.4 MB）。NR/FNR 计数、按文件（含空文件）重置 FNR、主输入 getline 共享游标、管道与协程不改计数等语义保持不变；缺失输入文件改为轮到它时才报错，报错后继续处理后续文件并退出 1。
- difftest 扩展：为 cp、mv、ls、du、find、tar 新增 32 条隔离 fixture 用例，除 stdout/退出码外比对执行后的文件树快照（路径、大小、只读位）；tar 创建类改为解包后比对条目。`du` 分配块数与 `mv -v` 措辞仍以 `$KnownDifferences` 记录，未放宽全局断言。

## 0.4.3 — xxd 移植与新增选项、awk 协程与 FNR、logger 事件日志(2026-11-03)

- 新增 2 个命令（sha3sum、xxd），命令总数 140 → 142（145 个包）：sha3sum 计算 SHA-3 摘要（--bits 支持 224/256/384/512，默认 512，多文件与 stdin，输出格式对齐 GNU）；xxd 提供十六进制转储与逆向。
- xxd 补齐 GNU 转储选项 -l、-s（绝对、+N 相对、-N 从末尾、0x）、-g（0=不分组）、-p、-i、-e（小端）、-u（大写）、-b（二进制位）、-a（autoskip 折叠）、-n（覆盖 -i 变量名）；默认列宽按 GNU（16，-i 12，-p 30，-b 6，-e 分组 4），冲突组合按 GNU 报错。地址、十六进制、-a 折叠与 -n 命名输出已用本机 GNU xxd 逐字节对照 46 组全部一致。
- 修复 xxd -r：只解析地址后的十六进制主体，ASCII 列不再参与，支持地址空洞零填充、`*` 折叠行、`-` 分组符与 CRLF 输入。
- awk 新增 `print |& cmd`、`cmd |& getline` / `cmd |& getline var` 协程：同一命令复用同一个 cmd /C 子进程（双向管道），close() 同时关闭读写两端并返回退出码，读入返回 1/0/-1，失败写 ERRNO。
- awk 新增 `cmd | getline` 与 `cmd | getline var`：同一命令字符串复用同一进程，close("cmd") 返回退出码并在下次使用时重启管道。
- awk 修正 FNR：按输入文件重置（打开成功的空文件也会归零），NR 仍跨文件累计；主输入 getline 与主循环共享同一记录游标，读取成功时同时递增 NR 与 FNR 且可跨文件续读；管道、协程与 `getline < file` 不改 NR/FNR，返回 0 或 -1 时都不改目标变量与 $0/NF；裸 `getline` 语句此前不实际读取，现按 GNU 语义执行。上述行为用本机 gawk 5.4.0 对照 17 组全部一致。
- logger 改为写入 Windows 应用程序事件日志：RegisterEventSourceW 注册来源、ReportEventW 逐行上报（固定事件 ID 1），-p 优先级映射 Error/Warning/Information；打开或上报失败、以及非 Windows 主机回退到 stderr 并打印原因，消息不会被静默丢弃。
- top 补 3 个集成测试（--version、单帧输出、进程表 20 行上限），并移除从未被使用的内存占用百分比返回值（输出不变）。
- 修复 chmod -R：递归遍历遇到 junction/符号链接按 GNU 语义忽略（不递归、不改属性）；此前会对条目本身应用模式并经 fs::metadata 跟随其目标，目标不可解析（如悬空 junction）时整个递归以 os error 3 中止退出 1。
- 修复 rm：删除前先用 ruxcmd-core 的 clear_readonly 清除只读属性（同 cp/unzip 的既有做法）——std 的 remove_file/remove_dir 不清只读位，`rm <只读文件>` 与 `chmod -R 444` 后 `rm -r` 的经典组合此前都会以 os error 5 中止；符号链接/junction 跳过清理以免波及目标。
- 文档修正：兼容性表把 iostat、mpstat、sar、w、pgrep、pkill、top 的验证状态改为行为测试；README 示例与 tag 引用同步到 0.4.3。
- 已知限制：awk 输入改为预加载（全量驻留内存），缺失输入文件在首条记录处理前即报错，BEGIN 阶段的 getline 不读主输入，ENDFILE 伪规则未实现；xxd 仍未实现 -C/-t 的 C 头定制、-o、-d、-E、-R 与 postscript，-s 不接受 `16k` 之类后缀；logger 在来源未注册消息 DLL 时无法显示消息正文。

## 0.4.2 — tree/envsubst/logger、awk getline(2026-10-31)

- awk getline：从文件读取行（getline < file、getline var < file），支持 while ((getline ...) > 0) 循环模式；close(file) 重置读取位置。
- 新增 3 个命令（tree/envsubst/logger），命令总数 137 → 140（143 个包）：tree（目录树，-a/-L/--charset=ascii）、envsubst（$VAR/${VAR} 环境变量替换，SHELL-FORMAT 过滤）、logger（标签消息输出到 stderr）。

## 0.4.1 — awk while/do-while、sed -f、dd conv 扩展(2026-10-31)

- awk：while 循环、do-while 循环、相邻值字符串拼接（`print "a" "b"` → `ab`，优先级紧于比较、松于加法）。
- sed：-f FILE 从文件读脚本。
- dd：conv=sync/noerror/ucase/lcase。
- 兼容性表双语同步；awk "仍不支持"列表缩减为 getline、协程 (|&)。

## 0.4.0 — 行为收口、命令扩展与 awk/sed 深化(2026-10-31)

### Round 24 awk 用户函数/重定向与 sed ERE(2026-10-31)

- awk 用户函数:function name(params) { body }、return 带值、递归(深度上限 512)、动态局部参数(调用后恢复);输出重定向:print/printf > 文件、>> 文件、| 命令(经 cmd /C)、/dev/stdout、/dev/stderr、close() 内建(GNU 语义:同一目标打开后持续写入)。
- sed -E/-r/--regexp-extended:切换 ERE 方案(共享转换器加开关);ERE 直传 Rust 正则语法(未转义 + ? {m,n} | ( ));反向引用在两种方言中均报错。
- 修复:expr_end 将函数调用括号误判为相邻值拼接;print 多参数在括号参数后丢失;statement_end 缺 Return 臂导致 if 内 return 值丢失;参数收集重写为显式深度扫描解决包装/无包装逗号深度混淆。

### Round 23 awk 数组与控制流(方向 3 收尾,2026-10-31)

- awk 支持关联数组:元素读写、delete arr[k] 与 delete arr、`k in arr`、for-in(键排序输出)、split 入数组(含默认 FS);三段式 for 循环与 i++/i-- 后缀;复合赋值 += -= *= /=(标量与数组元素)。
- 参数收集重写为显式深度扫描,修正三处历史缺陷:函数调用括号被当作相邻值拼接截断、print 的多参数在括号参数后丢失、包装式调用与无包装列表的逗号深度混淆。
- 已知差异更新:awk 仍不支持用户函数、重定向、相邻值字符串拼接(双语兼容性表同步)。

### Round 22 系统测试收尾与 dd(2026-10-31)

- 补齐 R17 遗留的四个系统命令行为测试：iostat（百分比范围/多采样）、mpstat（all 行/Average 汇总）、sar（时间戳报告/逐行百分比限定）、w（本地化启动头部）——至此仅冒烟清单清零，全部 137 个命令均有行为验证。
- 新增 dd（核心子集：if/of/bs/ibs/obs/count/skip/seek，GNU 尺寸后缀 c/w/b/K/KB/M/G；conv=notrunc，其余拒绝；records/copy 统计与 GNU 格式一致，实测向量对齐）；命令总数 136 → 137（140 个包）。
- 兼容性表双语与 README 计数同步；"仅冒烟"状态自此不复存在。

### Round 21 awk 解释器升级(方向 3 第二片,2026-10-31)

- awk 模式升级为 ERE(regex crate,编译期报错);求值器改为可变求值并新增内建函数:length(含裸形式)、substr(gawk 裁剪语义)、index、sprintf、toupper、tolower、sub/gsub(& 写回变量或 $0,字段重切分);printf 语句复用 printf crate 的完整 GNU 格式化器(crate 拆分为 lib+bin 共享)。
- 修复升级过程中暴露的三处执行缺陷:expr_end 将函数调用括号误判为相邻值拼接、sub/gsub 裸语句被跳过、Regex 字面量参数在求值中丢失。
- 已知差异更新:awk 仍不支持数组与 split、用户函数、重定向、相邻值拼接(双语兼容性表同步)。

### Round 20 差分收口与最后两个命令(2026-10-30)

- 新增 chgrp(LookupAccountNameW + SetNamedSecurityInfoW,复用 chown --group 语义,带 -R/-v/-f)与 b2sum(BLAKE2b,sha1sum 家族的 --tag/校验规则,-l 支持 8-512 位);命令总数 134 → 136(139 个包)。
- b2sum 引入 blake2 依赖;摘要与 GNU b2sum 实测一致(MSYS 对 stdin 的二进制标记差异沿用既有已知差异类)。
- 差分矩阵扩至 770 用例(echo -e 全集/printf 宽度精度复用/cmp fixture/factor/expand 列表/base32/sum/tsort/expr/sed 正则),并借差分修复三处对齐:sum -s 改为逐块 16 位一补数(此前整体折叠,实测向量 60108/65025/64515 全对齐)、base32 -w 0 不输出尾换行且 -d 拒绝非字母表数字、tsort 成环时向 stderr 报环成员并向 stdout 输出(与 GNU 一致,退出 1)。

### Round 19 sed 正则(方向 3 第一片,2026-10-30)

- sed 的地址与 s 命令升级为 POSIX BRE(含 GNU 扩展 \+、\?、\{m,n\}):替换端支持 & 与 \1-\9 分组引用、\n/\t/\r、g/N/p/i 标志;BRE 反向引用明确报错(记录为差异)。
- BRE 转换器提升为 ruxcmd-core 共享模块(此前在 expr 内部),expr 改为复用并锚定;regex 依赖移入 core。字节安全保持:非 UTF-8 数据按原始字节匹配与透传。
- 新增 15 个正则行为测试并用 GNU sed 做抽样差分(分组、&、g/N/p/i、区间、锚定地址、转义分隔符);sed 行为测试总数 50。
- 兼容性表双语 sed 行改写。

### Round 18 命令补齐(9 个新命令,2026-10-29)

- 新增 9 个命令,命令总数 125 → 134(137 个包):nproc、printenv、truncate、base32、sum、tsort、pathchk、hostid、expr。
- 全部参照本机 GNU coreutils 8.32 实测语义实现并附行为测试:expr 支持 i64 四则/比较/|&/锚定 BRE 匹配/substr/index/length 与 GNU 退出码(BRE 反向引用除外);sum 实现 BSD 与 SysV 两种算法(实测向量 36979/542);base32 按 RFC 4648 含 GNU 换行语义;truncate 支持 +/- 与 1024/1000 进制后缀;tsort 按 GNU 措辞报环与奇数 token;printenv 支持 -0 与任一缺失退出 1;pathchk 检查组件/路径长度与便携字符集;hostid 输出机器名散列。
- expr 引入 regex 依赖(BRE 转换);其余命令零新增依赖;冒烟清单 134 项,双语兼容性表与 README 计数同步。
- 明确不做并记录:chgrp(复用 chown 组逻辑,单独一轮)、dd、b2sum(需 BLAKE2 依赖)。

### Round 17 质量收口（行为测试覆盖 + P2 清偿，2026-10-29）

- 为 27 个"仅冒烟"命令补齐行为测试：clear/cmp/df/expand/factor/free/hexdump/kill/mkdir/mkfifo/readlink/realpath/reset/rev/seq/shred/strings/time/uname/who/whoami/zcat/printf/sleep/echo/which/whereis；iostat、mpstat、sar、w 仍为仅冒烟并在兼容性表声明。
- cmp 对齐 GNU：differ 消息改走 stdout（char N、line M），退出码 0/1/2，EOF 消息带引号与字节/行上下文（末字节为换行时省略 in），-l 八进制清单，单操作数与 stdin 比较，-l/-s 互斥拒绝；修复行号误用字节位置与打开失败退出码。
- printf 全量重写：宽度/精度/标志（- 0 + 空格 #）、d/i/u/o/x/X（base-0 解析、%u 回绕）、f/e/E/g/G（指数两位带符号、%g 去尾零）、%b（\c 截断）、%q、跨参数格式复用、无转换指令多余参数警告、未知指令报错退出 1、\a\b\e\f\v\'\? 转义与 coreutils 八位八进制规则；缺参 %f 输出 0.000000（GNU 一致）。
- echo 改为手工前导选项扫描（弃用 clap）：未知选项簇按操作数输出（`echo -x`、`echo --`），-e 补齐 \a\b\c（截断且无换行）\e\f\v\0NNN\xHH 全集。
- sleep 支持多操作数求和与小写 s/m/h/d 后缀；拒绝大写后缀与负数（GNU 语义），移除非 GNU 的 -v。
- which 改用 PATHEXT 探测（精确名优先、标准扩展后接自定义项、大小写不敏感去重），输出改为 PATH 书写形式不解析符号链接。
- whereis 移除硬编码 D:\develop\rust-tools 搜索路径，输出对齐 GNU（`cmd: path...`/裸 `cmd:`），新增 -b/-m/-s 与命中退出码。
- 其他修复：expand 字节安全化（CRLF/非 UTF-8/未终止末行透传）与 GNU tab 诊断（0 尺寸、升序）；factor 0 输出 `0:`；rev 按 util-linux 字符语义反转并对非法 UTF-8 字节回退；strings 对齐 binutils（TAB 可打印、-t 偏移 7 宽空格填充、8 位模式 raw 输出、-f 按书写路径）；shred 失败传播退出码 1；seq 接受选项后的负数操作数（`seq -w -3 0`）。
- 兼容性表双语逐条更新（验证状态与差异说明）；维持 125 个命令。

## 0.3.2 — Windows 文件属性与 ps 结构收敛（2026-10-25）

- 收敛 ps 结构：将仅供 ps 使用的 ps-core 合入 ps-bin，保留 ps/pgrep/pkill 共用的 ps-sys；以静态列描述表及四组格式化函数替代 40 个列类型和动态工厂，清理未使用的过滤/采样框架及内部字段。42 个列名/别名、格式、排序和树形输出保持原行为；当前 128 个包生成 125 个命令。

- 全部 125 个命令在普通编译时嵌入 Windows 版本资源：功能说明、产品/文件版本、发布者、版权、内部名称及原始文件名；套件版本集中管理，ps/vdir 身份分别正确，补齐缺失包描述。新增仅构建时使用的 ruxcmd-build，当前 128 个包仍生成 125 个命令。
- Verify 与打包新增原生版本属性门禁，ZIP 解压后再次验证并在 manifest 记录版本字段；新增真实 RC 特殊字符/空格路径/缓存刷新回归及缺失/过期/错误身份拒绝测试。不生成自签证书或修改系统信任。

## 0.3.1 — 桌面交互测试改为手动执行（2026-10-25）

- 将 open 的桌面交互测试统一改为显式手动执行，避免常规测试与发布验证启动真实窗口、因临时目录删除触发资源管理器“位置不可用”弹窗。

## 0.3.0 — 命令扩展、NUL 模式与流式处理（2026-10-25）

- 修复 top 行为测试遗漏系统态 CPU、在高内核负载主机上误报的问题；断言检查各列范围与百分比总和，并以失败样本锁定回归。
- Round 16 为 15 个已有命令增加 NUL 模式：sort/uniq/cut/paste/comm/shuf/head/tail 的 -z/--zero-terminated，basename/dirname/md5sum/sha1sum/sha256sum/sha512sum/cksum 的 -z/--zero。仍为 125 个命令，无新增依赖包版本。
- ruxcmd-core 的 ByteLines 增加可选分隔符，默认 LF/CRLF 规则保留；NUL 模式原样保留记录内 CR/LF 和非法 UTF-8。head/tail 不补末记录终止符；tail follow 在收到 NUL 前即可输出部分追加内容，并显式刷新初始输出。
- 同步修复：sort 不再将不同非法字节经有损字符串混同；sort/uniq 字段按 GNU 的 space/TAB/LF 分界（CR 保留）；uniq -f 不归一化剩余空白；comm 隐藏列后压缩 TAB；paste 解码分隔符转义、重复 - 共享 stdin，串行空输入仍输出终止符；tail 多文件标题增加 LF 间隔，-v 作用于 stdin。
- 校验和 NUL 输出不转义文件名，四种摘要拒绝 -z -c（exit 1）；md5sum/sha256sum 同步 Round 15 的 stdin 清单、精确文件名/GNU 转义/tag、FAILED 输出和读取错误传播修复。
- GNU 差分增加专用 NUL/未终止/NUL 二进制/空记录形态，保持旧矩阵形态不变；摘要 tag-zero 及四种摘要清单实际比较。GNU 具体选项能力探测先要求 --help 成功，旧 cksum 8.32 不支持 --zero 时只跳过该用例；不将进程初始化失败当作不支持选项。

- Round 15 新增 14 个命令：rmdir、link、unlink、mktemp、install、sha1sum、sha512sum、cksum、users、logname、nohup、watch、traceroute、vdir；当前 128 个包生成 125 个命令。ls 增加 -h，vdir 复用 ls 库并默认启用长格式。
- 新命令补齐行为测试、125 项冒烟清单、GNU 差分矩阵、双语兼容性说明及开发文档；差分测试为修改文件的命令创建两份隔离 fixture，比较 stdout、退出码及文件内容/目录状态。SHA 检查模式实际纳入差分，MSYS 生成格式、utmp 和长列表格式差异单列豁免。
- 收尾修复：install 单操作数误报成功、缺失 -D、目标链接别名覆盖；mktemp 改用 BCryptGenRandom，并补 -t、Windows 前缀/根路径与后缀校验；SHA 清单 stdin、精确文件名/GNU 转义/BSD tag、失败输出流及读取错误传播；cksum 隐式 stdin 不再输出文件名；nohup 使用 DETACHED_PROCESS、保留重定向流并明确警告 Job 脱离限制；traceroute 不再因连续三跳静默提前终止。
- Round 15 经维护者确认将 NUL 分隔的 -z 系列交给后续 Round 16，现由上述 Round 16 实现；仍不声明完整 GNU coreutils 覆盖，也不包含 winget 上架。

- 新增 GNU 差分测试框架 scripts/difftest.ps1:对 cat/head/tail/sort/uniq/tr/cut/paste/fold/wc/nl/grep/sed/od/seq/echo/printf 等 30+ 命令的 87 个用例矩阵,生成确定性随机输入(ASCII/CRLF/非 UTF-8/空/无尾换行/长行),与 Git 自带 GNU 工具逐字节比对 stdout 与退出码;支持种子复现、单命令筛选、已知差异豁免表与缺失工具自动跳过。
- 差分测试首轮发现并修复 5 个 GNU 对齐缺陷:seq 缺结尾换行;cut 无分隔符行未按 GNU 透传;nl 未给无尾换行输入补终止换行;sed 未保留缺失尾换行(纠正 0.2.0 前的"恒补 \n"错误假设);md5sum/sha256sum 文本模式改用 GNU 两空格分隔。MSYS 二进制模式校验和输出差异登记为已知差异。
- 更新 wc/sed/nl/cut/md5sum 兼容性表条目与相关测试锁定。

- 新增 ruxcmd-core 共享 crate(字节安全读行/路径净化/CPU 采样/只读清除),which/readlink/realpath、iostat/mpstat/sar/top/vmstat、grep/cut/sort、cp/unzip 的重复实现收敛(净删约 500 行);dirname/basename/head/tail/sed/fold 因语义差异保留本地实现(已记录)。
- 流式化:curl/wget 下载与 base64 编解码改为流式(300MB 下载内存峰值 342MB→11.5MB);nl/od/hexdump/strings/iconv 逐行/分块处理(1550 次随机差分验证与旧实现输出一致)。

## 0.2.0 — 缺陷修复与行为验证轮（2026-10-25）

- 完成第 12 轮 P1 缺陷修复(批次 A1-I1,约 45 个命令):进程工具原生化(pkill/pgrep/kill 弃用 tasklist/wmic/taskkill,复用 ps-sys;w/who 改 WTSEnumerateSessions);真实数据源(iostat/mpstat/sar/time/top/vmstat 的 GetSystemTimes、ps 的存活期 %CPU);文本工具字节安全化(grep/cut/sort/head/tail/sed/fold 的非 UTF-8 透传);sed 地址与 p/d/q 命令;awk 条件求值重写;网络工具语义修正(curl body-only/-d POST/-L/-f、wget 退出码 8、ping -t 持续语义);netstat -o PID、xargs -0 与分批、find GNU maxdepth 与单横线选项;chmod 符号模式、test POSIX 子集、env 命令执行;md5sum/sha256sum --tag/--status、od 全规格、iconv 严格 -f、numfmt 分隔符;tar -C/-O/--exclude、patch 多文件按计数解析等。
- 完成第 11 轮 P0 缺陷修复(48 任务,覆盖审查报告全部 50 项 P0):每个修复附行为回归测试,涉及命令的验证状态由"仅冒烟"升级为"冒烟 + 行为测试"。
  - 安全:tar/unzip 解包拒绝路径穿越条目(退出码 2);zip 参数经 PowerShell 单引号转义防注入;sync 防自递归并传播退出码。
  - 数据正确性:cat 字节透明化;base64 解码重写;touch 经 SetFileTime 真实更新时间戳;echo 默认不解释转义;printf 实现格式串转义;bc 求值器重写。
  - 进程控制:nice 改用优先级类并透传退出码;timeout 轮询子进程、超时返回 124 并实现 -k。
  - 伪造数据清除:nslookup 经 DnsQuery_W 真实查询 MX/TXT/CNAME;id/groups 取自进程令牌;ps %MEM 用真实总内存;chown --group 正确设置组;tty 改用 IsTerminal;vmstat 逐帧采样。
  - 输出与路径语义:which/readlink/realpath/dirname/basename 输出净化;ls 着色下长格式列不丢失;stat -c 对齐 GNU;sort 实现 -k/-t;diff 退出码与行号;uniq/tac/nl/wc/tee/tail/split/csplit/column/pr/shuf/du/grep/file 语义修复。
  - 附加:df/du/free/pr/whoami/sort 的 -h 与 help 冲突(六命令 debug 构建启动即 panic)修复;test 的 clap 参数名冲突修复。
- 完成全部 111 个命令的源码级代码审查(报告 docs/command-review-2026-10-07.md):50 个命令存在 P0 级缺陷,报告给出逐命令的功能/移植完整性/技术评估与 P0-P2 分级。
- 制定第 11 轮 P0 缺陷修复设计与计划(specs/2026-10-07-p0-remediation-design.md、plans/2026-10-07-p0-remediation.md):安全(zip slip/命令注入/自递归)、数据损坏、panic、伪造数据清除四大类,48 个任务。
- 清理提交进仓库的手工测试杂物(29 个 crate 根目录的 test_*/checksum/fixture 文件与 ls 的 .codegraph/reasonix.toml),.codegraph 加入 .gitignore。
- 新增全仓库 MIT 许可证（LICENSE）、贡献者行为准则（CODE_OF_CONDUCT.md）与安全策略（SECURITY.md）。
- 根 Cargo.toml 增加 `[workspace.package]`，113 个 crate 统一声明 license/repository 元数据。
- 贡献规范、开发说明、命令兼容性表及脚本使用说明提供中英双语版本，中文版使用 `-zh.md` 后缀；两份 README 更新徽章、文档链接与许可证章节。
- 清理迁移遗留的根目录构建产物（游离 `ls/` 目录、`build-output.log`、`rust.html`）。
- 用 PowerShell 构建入口替代 build-all.bat，支持任务、单包、Debug/Release、离线、并行数及只读预览。
- 增加本地与 GitHub 发布入口，复用清单、校验和及解压验证，拒绝冲突 tag 和覆盖已有 Release。
- CI 与 Release workflow 复用统一入口，补充入口失败路径测试及中英文使用说明。

## 0.1.0 — 基础设施基线

- 将 111 个 Windows 命令归入根 Cargo workspace，固定工具链并统一依赖锁定。
- 增加代表性行为测试及每个命令的发布冒烟策略。
- 增加带源码信息、PE 依赖记录、SHA-256 及解压验证的 ZIP 打包流程。
- 增加 Windows CI、手动发行 artifact workflow 及贡献文档。
- 修复 cp/mv 源文件参数声明导致的 debug 启动断言，以及 grep 普通模式未匹配时的退出码。
- 将 csplit 的溢出边界检查改为 checked_add，消除 Clippy 阻断。

此版本建立发布基线；不代表所有命令已经完整兼容 GNU/POSIX 行为。
