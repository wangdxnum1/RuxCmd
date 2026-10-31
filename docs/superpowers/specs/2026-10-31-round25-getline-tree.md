# Round 25: awk getline + 新命令 tree/envsubst/logger

日期：2026-10-31。基线：main（v0.4.1 发布后）。

## 目标

1. awk `getline`：从文件/stdin 读取行（最常用的 awk 剩余特性）
2. 新命令 **tree**（目录树）、**envsubst**（环境变量替换）、**logger**（Windows 事件日志）

## awk getline 设计

### 支持的形态

| 形式 | 语义 |
|---|---|
| `getline` | 从当前输入读下一行到 $0 |
| `getline var` | 从当前输入读下一行到 var |
| `getline < file` | 从 file 读下一行到 $0 |
| `getline var < file` | 从 file 读下一行到 var |

返回值：成功 1、EOF 0、错误 -1。

### 架构

- 每文件状态存储在 thread_local `GETLINE_SOURCES: HashMap<String, (Vec<String>, usize)>`，按路径缓存全文与行偏移
- 裸 `getline` 需要访问主输入流：process_input 将行存入 thread_local `MAIN_INPUT`，通过 NR 定位
- `getline` 在 eval_tokens 中处理：识别 Getline token，解析后续 var/< file，返回 Num(1/0/-1)

### 限制（声明差异）

- `cmd | getline` 管道输入形式不支持
- `getline` 关闭文件（`close(file)`）后重置读取位置 ✓（close 清除缓存）

## tree 设计

```
tree [directory] [-a] [-L level] [--charset=ascii]
```
- 递归遍历目录，Unicode 树枝（├── └── │）或 ASCII（|-- `-- |--）
- -a 显示隐藏文件，-L level 限制深度
- 每层排序（目录优先，同名按字典序）
- 输出统计：N directories, M files

## envsubst 设计

```
envsubst [SHELL-FORMAT]
```
- 从 stdin 读入，替换 `$VAR` 和 `${VAR}` 为环境变量值
- 有 SHELL-FORMAT 参数时只替换列出的变量
- 未知变量替换为空串（与 GNU envsubst 一致）

## logger 设计

```
logger [-s] [-t tag] [message]
```
- `-s` 同时输出到 stderr
- `-t tag` 设置标识前缀
- 无参数时从 stdin 读
- Windows 实现：输出到 stderr + 可选写 Windows Event Log（ReportEvent）为后续增强
- 当前版本简化为格式化输出（与 GNU logger 的 syslog 语义差异声明）
