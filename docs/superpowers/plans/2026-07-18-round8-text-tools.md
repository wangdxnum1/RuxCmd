# Round 8 实现计划：文本处理工具集

## 任务总览

| 任务 | 工具 | 内容 |
|------|------|------|
| Task 0 | 脚手架 | 创建 awk、iconv、hexdump 三个新 crate 的基础结构；更新 build-all.bat |
| Task 1 | awk | 实现 awk 文本处理语言（支持 print、printf、if、for、变量等） |
| Task 2 | iconv | 实现字符编码转换（UTF-8/GBK/GB2312/UTF-16等） |
| Task 3 | od (增强) | 增强现有 od：新增 -j、-N、-w、-A n 等选项 |
| Task 4 | hexdump | 实现 hexdump 十六进制转储工具（-C 模式等） |
| Task 5 | strings (增强) | 增强现有 strings：新增 -f、-t 选项 |
| Task 6 | 构建验证 | 运行 build-all.bat 验证所有工具 |

---

## Task 0: 脚手架

**目标**：创建新工具的基础结构，更新构建脚本

**步骤**：
1. 创建 `awk/Cargo.toml`、`awk/src/cli.rs`、`awk/src/main.rs`
2. 创建 `iconv/Cargo.toml`、`iconv/src/cli.rs`、`iconv/src/main.rs`
3. 创建 `hexdump/Cargo.toml`、`hexdump/src/cli.rs`、`hexdump/src/main.rs`
4. 更新 `build-all.bat` 添加 `awk iconv hexdump`

---

## Task 1: awk

**目标**：实现 awk 文本处理语言

**CLI**：
```
awk [OPTIONS] 'PATTERN { ACTION }' [FILE]...
  -F, --field-separator=SEP   字段分隔符
  -v, --assign=VAR=VALUE      设置变量
  -f, --file=FILE             从文件读取程序
  -v, --version               版本
```

**实现要点**：
- 词法分析器：tokenize awk 程序
- 语法分析器：解析模式-动作对
- 执行引擎：逐行处理，维护 NR、NF、$0、$1..$NF
- 支持 print、printf、if、for、赋值
- 默认动作 { print }

---

## Task 2: iconv

**目标**：实现字符编码转换

**CLI**：
```
iconv [OPTIONS] -f FROM -t TO [FILE]...
  -f, --from-code=NAME     源编码
  -t, --to-code=NAME       目标编码
  -o, --output=FILE        输出文件
  -l, --list               列出支持的编码
  -v, --version            版本
```

**实现要点**：
- 使用 encoding_rs 进行转换
- 支持 UTF-8、GBK、GB2312、UTF-16、ISO-8859-1
- -l 列出支持编码
- 错误处理：非法字节替换

---

## Task 3: od (增强)

**目标**：增强现有 od 工具

**新增选项**：
- `-j N`：跳过前 N 字节
- `-N N`：最多读取 N 字节
- `-w N`：每行字节数
- `-A n`：无地址模式

**修改文件**：
- `od/src/cli.rs`：新增参数
- `od/src/main.rs`：实现跳过和限制逻辑

---

## Task 4: hexdump

**目标**：实现 hexdump 工具

**CLI**：
```
hexdump [OPTIONS] [FILE]
  -C, --canonical             标准十六进制+ASCII格式
  -d, --decimal               双字节十进制
  -o, --octal                 双字节八进制
  -x, --hexadecimal           双字节十六进制
  -c, --ascii                 单字节字符
  -s, --skip=N                跳过前 N 字节
  -n, --length=N              最多显示 N 字节
  -v, --version               版本
```

**输出格式**：
```
00000000  48 65 6c 6c 6f 20 57 6f  72 6c 64 21 0a 00        |Hello World!..|
```

---

## Task 5: strings (增强)

**目标**：增强现有 strings 工具

**新增选项**：
- `-f`：多文件时前缀文件名
- `-t RADIX`：输出字符串位置

**修改文件**：
- `strings/src/cli.rs`：新增参数
- `strings/src/main.rs`：实现文件名前缀和位置输出

---

## Task 6: 构建验证

**步骤**：
1. `cargo check` 所有新工具
2. 运行 `build-all.bat`
3. 验证 `D:\develop\rust-tools` 有新 exe
4. 简单冒烟测试
5. 更新进度文档