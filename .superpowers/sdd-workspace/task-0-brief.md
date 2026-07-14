# Task 0: Scaffold Build Pipeline (build-all.bat)

**Files:**
- Modify: `build-all.bat:16`

**Interfaces:**
- Produces: PROJECTS 列表包含 10 个新工具名，后续批处理流程可用。

- [ ] **Step 1: 查看 build-all.bat 第 16 行内容**

Read: `build-all.bat` 第 16 行，确认为 `set "PROJECTS=..."` 一行。

- [ ] **Step 2: 在 PROJECTS 末尾追加 10 个工具名**

在已有的 `whereis` 后追加（保留顺序）：

```
set "PROJECTS=cat ls open rm touch mkdir cp mv head tail wc grep date which sort cut find du df kill echo ln whoami chmod chown uname uptime env diff sed uniq tee xargs basename dirname tr cmp tar rev split paste nl file md5sum sha256sum stat readlink realpath seq yes sleep id free fmt fold comm join who w hostname groups gunzip unzip zcat patch od strings tty mkfifo true false test printf cal clear reset pkill pgrep shred sync bc zip time timeout nice whereis base64 expand unexpand tac column shuf csplit pr numfmt factor"
```

- [ ] **Step 3: 保存后运行批处理的头部检查**

Run:
```
cd /d d:\Work\rust && findstr /c:"base64 expand unexpand" build-all.bat
```
Expected: 输出完整 PROJECTS 行并包含所有 10 个新名字。

- [ ] **Step 4: Commit build-all.bat 修改**

```
cd /d d:\Work\rust
git add build-all.bat
git commit -m "chore(build): add round 7 tools to build-all.bat PROJECTS list"
```
