# Task 7: 最终 build-all 全量构建 + 88 exe 清点 + 10 新工具 smoke + overall commit

Round 7 最后一个任务。目标：
1. 运行 build-all.bat，它会对所有 PROJECTS 里的 crate build release --offline 并 xcopy exe 到 D:\develop\rust-tools。
2. 清点 D:\develop\rust-tools 下的 .exe 文件数：应为 88 个（Rounds 1-6 = 78 个；Round 7 新增 10 个 = 88）。
3. 在 D:\develop\rust-tools 目录下为 Round 7 新增的 10 个工具各执行 1 条 1-line smoke（快速验证功能）。
4. 若有任何新/改动文件需提交（如 build-all.bat, 报告文件，progress.md 等已有其他 commit 过——本任务只提交本任务产生的新改动，如果没有则 overall commit 可选跳过）。

## Step 1: 记录 BASE（当前 HEAD）

```
cd /d d:\Work\rust
git rev-parse HEAD > .superpowers\sdd\base-t7.txt
git status --short
```

## Step 2: 运行 build-all.bat 全量构建

```
cd /d d:\Work\rust
build-all.bat
```

Expected: 整个脚本无 error；末尾应该有 echo 说 DONE。所有 crate build 成功。

## Step 3: 清点 exe 数量（88 个目标）

PowerShell 里执行：
```
$exes = Get-ChildItem D:\develop\rust-tools -Filter *.exe | Sort-Object Name
Write-Host "Count: $($exes.Count)"
$exes | ForEach-Object { Write-Host "  $($_.Name)  $($_.Length) bytes" }
```
Expected: Count == 88。Round 7 新增 10 个 name 精确如下（都要出现）：
  base64.exe, column.exe, csplit.exe, expand.exe, factor.exe, numfmt.exe, pr.exe, shuf.exe, tac.exe, unexpand.exe

## Step 4: Round 7 新增 10 个工具各 1-line smoke（在 D:\develop\rust-tools 目录下执行）

```
cd /d D:\develop\rust-tools
```

① factor
```
.\factor.exe 12 1001
```
Expected: 12: 2 2 3\n1001: 7 11 13\n（或类似正确分解）。

② base64
```
echo Hello | .\base64.exe
```
Expected: 编码后首段以 SGVsbG8（不区分换行）开头。

③ expand
```
cmd /c "echo a<TAB>b<TAB>c" | .\expand.exe -t 8
```
Expected: Tab 被替换成空格，总长度对齐。

④ unexpand
```
echo "        x" | .\unexpand.exe -t 8
```
Expected: 8 空格被替换为一个 \t（可以管道给 expand.exe 回环验证）。

⑤ tac
```
cmd /c "echo 1 & echo 2 & echo 3" | .\tac.exe
```
Expected: 3 2 1 倒序。

⑥ column
```
cmd /c "echo 1,22,333 & echo a,bb,ccc" | .\column.exe -t -s "," -o "|"
```
Expected: 输出有两行列对齐 + `|` 分隔。

⑦ shuf
```
.\shuf.exe -i 1-5 --random-source "x"
```
Expected: 5 行（每行 1..5 一个不重复整数，同一 seed 稳定）。

⑧ csplit
```
Set-Location $env:TEMP; Remove-Item xx??,testcs.txt -ErrorAction SilentlyContinue
Set-Content -Path testcs.txt -Value "first`n---`nsecond`n---`nthird" -NoNewline
& D:\develop\rust-tools\csplit.exe -k testcs.txt "/---/"
Get-ChildItem xx?? | ForEach-Object { Write-Host $_.Name (Get-Content $_.FullName | Measure-Object -Line).Lines }
Remove-Item xx??,testcs.txt
Set-Location D:\develop\rust-tools
```
Expected: xx00 (1 行 first), xx01（含 --- second 2 行左右）, xx02（--- third 2 行左右）。

⑨ pr
```
cmd /c "echo a & echo b & echo c" | .\pr.exe -n " " -h smk
```
Expected: 含页眉 (日期 + 居中 "smk" + 等号分隔) + 行号（ N ...）。

⑩ numfmt
```
.\numfmt.exe --to=iec-i 1536 ; .\numfmt.exe --to=iec 1536 ; .\numfmt.exe --to=si 1500
```
Expected: 三行：1.5Ki / 1.5K / 1.5K（或类似数字）。

## Step 5: overall commit（如有本 Task 产生的改动文件）

回到 d:\Work\rust:
```
cd /d d:\Work\rust
git status --short
```
- 如果有 uncommitted changes：`git add -A && git commit -m "chore(round7): final build-all smoke 88 exes count verified; no code changes"`
- 如无 uncommitted changes：跳过 commit，在报告中注明 "no new files/modifications this task; no commit needed"。

## Constraints

- 严格按 Step 做；所有实际输出要全部记录到报告里（不要缩略）。
- 任一步骤 failure，立刻标记 DONE_WITH_CONCERNS，并把具体错误放进报告与返回摘要。
- .superpowers/sdd/progress.md 已由 Controller 维护，不要改它。
- 不改动任何 crate 源文件。

## Report

详细全步骤输出写入：d:\Work\rust\.superpowers\sdd-workspace\task-7-report.md

返回短摘要（15 行内）：
- Status: DONE / DONE_WITH_CONCERNS / BLOCKED
- 如果有 overall commit: short SHA + subject；否则 "(no commit)"
- 1-line summary（build-all OK? 88/88 count? 10/10 smokes? 具体 failed 个数是多少）
- Concerns if any
- Report 路径
