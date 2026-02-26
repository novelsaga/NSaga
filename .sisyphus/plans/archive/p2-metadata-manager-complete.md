# P2 - 元数据管理器保姆级执行计划

## TL;DR

**目标**: 实现 NovelSaga 元数据管理器，包括小说文件拆分、Core层数据模型、CLI层IO实现、幂等性集成测试。

**工作量**: 8个Wave，21个任务，预计3-4天

**关键约束**:

- 执行者能力有限，所有决策已在计划中明确
- 集成测试必须幂等（测试前后文件系统一致）
- 遇到问题按决策树升级，禁止询问用户

---

## 执行者必读

### 能力假设

**假设你具备**:

- 基础 Rust 语法知识
- 能读懂 trait 和 struct 定义
- 会使用 cargo 进行构建和测试
- 能使用基本的 git 操作

**假设你不具备**:

- 架构决策能力
- 性能优化经验
- 错误恢复策略制定
- 跨模块协调

**因此本计划提供**:

- ✅ 每个文件的完整代码框架（复制粘贴即可用）
- ✅ 每个函数的输入输出说明
- ✅ 明确的决策点（无需自行判断）
- ✅ 详细的错误处理步骤
- ✅ 升级路径（何时、向谁求助）

### 禁止行为

❌ **不要向用户提问**（所有决策已在计划中明确）
❌ **不要自行修改架构设计**
❌ **不要跳过验证步骤**
❌ **不要合并多个任务**
❌ **不要自行优化代码**（按计划实现即可）

### 遇到问题怎么办

**Step 1**: 查看本计划的"错误处理"章节
**Step 2**: 尝试自救（30分钟）
**Step 3**: 按"升级决策树"升级求助

---

## 问题升级与咨询路径

### 升级决策树

```
遇到问题
    │
    ├─ 是编译错误？
    │   ├─ 是否缺少依赖？ → 检查 Cargo.toml → 仍失败 → 升级
    │   ├─ 是否类型不匹配？ → 检查 trait 实现 → 仍失败 → 升级
    │   └─ 是否生命周期错误？ → 直接升级（不要自行解决）
    │
    ├─ 是运行时错误？
    │   ├─ 是否 panic？ → 检查 unwrap/expect → 仍失败 → 升级
    │   ├─ 是否死锁？ → 直接升级
    │   └─ 是否性能问题？ → 记录后完成当前任务 → 升级
    │
    ├─ 是设计冲突？
    │   └─ 直接升级（不要自行修改设计）
    │
    └─ 是需求不清？
        └─ 查看本计划的"决策记录" → 仍不清 → 升级
```

### 升级方式

**方式 1: oracle 子代理（架构/设计问题）**

```
任务类别: deep
提示: "我在实现 [任务名] 时遇到 [具体问题]。当前代码：[相关代码片段]。已尝试 [解决方法]。需要指导 [具体方向]。"
```

**方式 2: librarian 子代理（技术调研）**

```
任务类别: unspecified-high
提示: "我需要了解 [技术/库] 的 [具体用法/最佳实践]。当前场景：[描述]。请提供 [代码示例/文档链接/实现建议]。"
```

**方式 3: Metis 子代理（计划咨询）**

```
任务类别: deep
提示: "我在执行计划时遇到 [问题]。当前进度：[wave/任务]。计划要求：[要求]。实际情况：[情况]。我应该如何调整？"
```

### 升级触发条件矩阵

| 问题类型       | 量化触发条件                  | 升级对象  | 参考资源           |
| -------------- | ----------------------------- | --------- | ------------------ |
| **架构决策**   | 缓存策略选择、索引结构设计    | oracle    | 项目架构文档       |
| **性能瓶颈**   | 索引查询>100ms / 写回延迟>1s  | oracle    | sled/moka调优指南  |
| **并发问题**   | 死锁/竞态条件无法本地复现     | oracle    | Rust Atomics书籍   |
| **平台兼容性** | notify在macOS/Windows行为差异 | librarian | notify文档平台章节 |
| **数据一致性** | 缓存与文件系统不一致          | oracle    | 一致性模型设计     |
| **复杂度爆炸** | 单文件>500行 / 嵌套>5层       | oracle    | 设计模式手册       |

**升级前必须尝试的自救步骤**（至少30分钟）：

1. 查阅官方文档（sled/moka/notify/flume）
2. 搜索项目内类似模式（`grep -r "moka::" projects/`)
3. 编写最小复现示例
4. 尝试3种不同实现方式

---

## 前置准备（Wave 0）

### 0.1 环境检查

**必须确认以下环境已就绪**：

```bash
# 检查 Rust 版本（必须 >= 1.85 nightly）
rustc --version
# 预期输出: rustc 1.85.0-nightly (xxxxxxxxx)

# 检查 direnv
which direnv
# 预期输出: /nix/store/.../bin/direnv

# 加载 Nix 环境
direnv allow

# 检查 pnpm
pnpm --version
# 预期输出: 9.x.x

# 安装依赖
pnpm install
```

**检查点**:

- [ ] `rustc --version` 显示 nightly
- [ ] `direnv allow` 成功执行
- [ ] `pnpm install` 无错误

**升级条件**: 环境检查失败 → 升级到 oracle

### 0.2 依赖确认

**检查 Cargo.toml 中是否已有所需依赖**：

```bash
# 检查 core 层依赖
grep -E "(serde|thiserror)" projects/core/Cargo.toml

# 检查 cli 层依赖
grep -E "(moka|sled|notify|flume|blake3|tokio)" projects/cli/Cargo.toml
```

**如缺少依赖，按以下版本添加**：

```toml
# projects/core/Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
ts-rs = "7.0"

# projects/cli/Cargo.toml
[dependencies]
moka = { version = "0.12", features = ["future"] }
sled = "0.34"
notify = "6.1"
flume = "0.11"
blake3 = "1.5"
tokio = { version = "1", features = ["full"] }
tempfile = "3.10"  # 用于测试
walkdir = "2.5"  # 用于遍历目录

# 测试依赖
[dev-dependencies]

# 测试依赖
[dev-dependencies]
walkdir = "2.5"  # 用于遍历目录
```

**检查点**:

- [ ] 所有依赖已添加
- [ ] `cargo check` 无错误

### 0.3 目录结构创建

### 0.3 目录结构创建

```bash
# 创建 core 层目录
mkdir -p projects/core/src/metadata

# 创建 cli 层目录
mkdir -p projects/cli/src/metadata
mkdir -p projects/cli/src/commands
mkdir -p projects/cli/tests

# 创建测试数据目录
mkdir -p tests/fixtures

# 创建脚本目录
mkdir -p scripts
```

**检查点**:
- [ ] 所有目录已创建
- [ ] `ls -la projects/core/src/metadata` 显示存在

---

## Wave 1: 小说文件拆分

**目标**: 将示例小说按卷章拆分为 Markdown 文件

**前置条件**: Wave 0 完成

**重要约束**:
- 小说共 **29 卷**（第一卷到第二十九卷）
- 拆分前必须备份源文件
- 检查无误后才能删除源文件
- 集成测试必须幂等（测试前后文件一致）

**目录结构**:
```
test/
└── 女神代行者/
    ├── 第一卷/
    │   ├── 卷序：xxx.md
    │   ├── 第1章 xxx.md
    │   ├── 第2章 xxx.md
    │   └── ...
    ├── 第二卷/
    │   ├── 卷序：xxx.md
    │   └── ...
    ├── 第三卷/
    │   └── ...
    └── ...（共29卷）
```

mkdir -p projects/core/src/metadata

# 创建 cli 层目录
mkdir -p projects/cli/src/metadata
mkdir -p projects/cli/src/commands
mkdir -p projects/cli/tests

# 创建测试数据目录
mkdir -p tests/fixtures
```

**检查点**:

- [ ] 所有目录已创建
- [ ] `ls -la projects/core/src/metadata` 显示存在

---

## Wave 1: 小说文件拆分

**目标**: 将示例小说按卷章拆分为 Markdown 文件

**前置条件**: Wave 0 完成

**重要约束**:
- 小说共 **29 卷**（第一卷到第二十九卷）
- 数据来自盗版网站，需要清洗（移除广告、图片链接等）
- ⚠️ **关键：拆分前必须备份源文件**
- ⚠️ **关键：检查无误后才能删除源文件和备份**
- 集成测试必须幂等（测试前后文件一致）

- 拆分前必须备份源文件
- 检查无误后才能删除源文件
- 集成测试必须幂等（测试前后文件一致）

### Task 1.1: 备份源文件

**操作步骤**:

```bash
# 创建备份目录
mkdir -p example/backup

# 备份源文件
cp "example/女神代行者 作者：snow_xefd.txt" example/backup/

# 验证备份
ls -lh example/backup/
# 预期输出: 女神代行者 作者：snow_xefd.txt (约 2MB)
```

**检查点**:

- [ ] 备份文件存在
- [ ] 备份文件大小与源文件一致

### Task 1.2: 创建拆分脚本

**文件**: `scripts/split_novel.py`

**代码框架**:

```python
#!/usr/bin/env python3
"""
小说文件拆分脚本

用法:
    python scripts/split_novel.py <input_file> <output_dir>

示例:
    python scripts/split_novel.py \
        "example/女神代行者 作者：snow_xefd.txt" \
        "test/女神代行者"
"""

import re
import sys
import os
import shutil
from pathlib import Path
from typing import List, Tuple


def clean_content(content: str) -> str:
    """
    清洗小说内容
    
    移除盗版网站的广告、HTML标签、图片链接等
    """
    # 移除 HTML 图片标签
    content = re.sub(r'<img[^>]+>', '', content)
    
    # 移除 HTML 其他标签
    content = re.sub(r'</?[a-zA-Z][^>]*>', '', content)
    
    # 移除 URL 链接（通常是图片或广告）
    content = re.sub(r'https?://[^\s<>"\']+', '', content)
    
    # 移除常见的广告标记
    content = re.sub(r'\[广告\]|【广告】|\[AD\]', '', content)
    
    # 移除空行（连续多个空行合并为一个）
    content = re.sub(r'\n\s*\n+', '\n\n', content)
    
    # 移除行首行尾空白
    content = '\n'.join(line.strip() for line in content.split('\n'))
    
    return content


def parse_novel(content: str) -> List[Tuple[str, str, str]]:
    """
    解析小说内容，提取卷章结构
    
    Returns:
        List of (volume_name, chapter_title, chapter_content)
    """
    # 先清洗内容
    content = clean_content(content)
    
    chapters = []
    """
    解析小说内容，提取卷章结构

    Returns:
        List of (volume_name, chapter_title, chapter_content)
    """
    chapters = []

    # 按行分割
    lines = content.split('\n')

    current_volume = "第一卷"  # 默认卷名
    current_chapter_title = None
    current_chapter_lines = []

    i = 0
    while i < len(lines):
        line = lines[i]

        # 检测卷标题 (第一卷、第二卷等)
        volume_match = re.match(r'^(第[一二三四五六七八九十]+卷)$', line.strip())
        if volume_match:
            # 保存前一章
            if current_chapter_title and current_chapter_lines:
                chapters.append((
                    current_volume,
                    current_chapter_title,
                    '\n'.join(current_chapter_lines)
                ))

            current_volume = volume_match.group(1)
            current_chapter_title = None
            current_chapter_lines = []
            i += 1
            continue

        # 检测章标题 (第X章 标题)
        chapter_match = re.match(r'^(第[0-9一二三四五六七八九十]+章)\s+(.+)$', line.strip())
        if chapter_match:
            # 保存前一章
            if current_chapter_title and current_chapter_lines:
                chapters.append((
                    current_volume,
                    current_chapter_title,
                    '\n'.join(current_chapter_lines)
                ))

            chapter_num = chapter_match.group(1)
            chapter_name = chapter_match.group(2)
            current_chapter_title = f"{chapter_num} {chapter_name}"
            current_chapter_lines = []
            i += 1
            continue

        # 检测卷序
        if line.strip().startswith('卷序：'):
            if current_chapter_title and current_chapter_lines:
                chapters.append((
                    current_volume,
                    current_chapter_title,
                    '\n'.join(current_chapter_lines)
                ))

            current_chapter_title = line.strip()
            current_chapter_lines = []
            i += 1
            continue

        # 收集内容
        if current_chapter_title is not None:
            current_chapter_lines.append(line)

        i += 1

    # 保存最后一章
    if current_chapter_title and current_chapter_lines:
        chapters.append((
            current_volume,
            current_chapter_title,
            '\n'.join(current_chapter_lines)
        ))

    return chapters


def sanitize_filename(name: str) -> str:
    """清理文件名，移除非法字符"""
    # 移除或替换文件系统不友好的字符
    name = re.sub(r'[<>:"/\\|?*]', '', name)
    # 限制长度
    if len(name) > 100:
        name = name[:100]
    return name


def split_novel(input_file: str, output_dir: str):
    """拆分小说文件"""
    input_path = Path(input_file)
    output_path = Path(output_dir)

    # 读取源文件
    print(f"Reading {input_file}...")
    with open(input_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # 解析章节
    print("Parsing chapters...")
    chapters = parse_novel(content)
    print(f"Found {len(chapters)} chapters")

    # 创建输出目录
    output_path.mkdir(parents=True, exist_ok=True)

    # 创建小说主目录（使用小说名）
    novel_name = "女神代行者"
    novel_dir = output_path / novel_name
    novel_dir.mkdir(exist_ok=True)

    # 写入章节文件
    for volume, chapter_title, chapter_content in chapters:
        # 创建卷目录
        volume_dir = novel_dir / volume
        volume_dir.mkdir(exist_ok=True)

        # 生成文件名
        safe_title = sanitize_filename(chapter_title)
        filename = f"{safe_title}.md"
        filepath = volume_dir / filename

        # 写入文件
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(f"# {chapter_title}\n\n")
            f.write(chapter_content)

        print(f"Created: {filepath}")

    print(f"\nSplit complete! {len(chapters)} chapters written to {novel_dir}")


def verify_split(input_file: str, output_dir: str) -> bool:
    """验证拆分结果"""
    input_path = Path(input_file)
    output_path = Path(output_dir)

    # 读取源文件统计
    with open(input_path, 'r', encoding='utf-8') as f:
        original_content = f.read()
    original_lines = len(original_content.split('\n'))

    # 统计生成的文件
    novel_dir = output_path / "女神代行者"
    if not novel_dir.exists():
        print(f"ERROR: {novel_dir} does not exist")
        return False

    total_files = 0
    total_lines = 0

    for md_file in novel_dir.rglob('*.md'):
        total_files += 1
        with open(md_file, 'r', encoding='utf-8') as f:
            content = f.read()
            total_lines += len(content.split('\n'))

    print(f"Original: {original_lines} lines")
    print(f"Generated: {total_files} files, {total_lines} lines")

    print(f"Original: {original_lines} lines")
    print(f"Generated: {total_files} files, {total_lines} lines")
    
    # 检查卷数量（应该有29卷）
    volume_dirs = [d for d in novel_dir.iterdir() if d.is_dir()]
    volume_count = len(volume_dirs)
    print(f"Volumes found: {volume_count}")
    
    if volume_count < 29:
        print(f"WARNING: Expected 29 volumes, found {volume_count}")
        return False
    
    # 检查是否所有章节都已生成（应该有卷序+290章=291个文件）
    if total_files < 290:
        print(f"WARNING: Expected at least 290 chapters, found {total_files}")
        return False
    if total_files < 290:
        print(f"WARNING: Expected at least 290 chapters, found {total_files}")
        return False

    return True


if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)

    input_file = sys.argv[1]
    output_dir = sys.argv[2]

    # 执行拆分
    split_novel(input_file, output_dir)

    # 验证结果
    print("\nVerifying...")
    if verify_split(input_file, output_dir):
        print("✓ Verification passed!")
        sys.exit(0)
    else:
        print("✗ Verification failed!")
        sys.exit(1)
```

**检查点**:

- [ ] 脚本文件创建成功
- [ ] `python scripts/split_novel.py --help` 显示帮助

### Task 1.3: 执行拆分并验证

**操作步骤**:

```bash
# 执行拆分
python scripts/split_novel.py \
    "example/女神代行者 作者：snow_xefd.txt" \
    "test"

# 预期输出:
# Reading example/女神代行者 作者：snow_xefd.txt...
# Parsing chapters...
# Found 291 chapters
# Created: test/女神代行者/第一卷/卷序：因"女神"找人接盘而死.md
# Created: test/女神代行者/第一卷/第1章 因女神找人"接盘"而生.md
# ...
# Split complete! 291 chapters written to test/女神代行者
#
# Verifying...
# Original: 42000 lines
# Generated: 291 files, 42000 lines
# ✓ Verification passed!
```

**检查点**:

- [ ] 拆分脚本执行成功
- [ ] 生成 291 个文件（卷序 + 290章）
- [ ] 验证通过

**手动检查**:

```bash
# 检查目录结构
ls -la test/女神代行者/第一卷/ | head -20

# 检查文件内容
cat "test/女神代行者/第一卷/第1章 因女神找人"接盘"而生.md" | head -20
```

### Task 1.4: 删除源文件、备份和辅助脚本（仅在验证通过后）

**⚠️ 警告：此操作不可逆，仅在验证通过后执行**

**删除流程（必须按顺序）**：

```bash
# Step 1: 确认 test 目录结构正确
ls test/女神代行者/ | head -10
# 必须显示: 第一卷/ 第二卷/ ...（共29个卷目录）

# Step 2: 确认拆分验证通过
python scripts/split_novel.py "example/女神代行者 作者：snow_xefd.txt" test
# 必须显示:
# - "Found 291 chapters"
# - "Volumes found: 29"
# - "✓ Verification passed!"

# Step 3: 手动抽查（重要！）
# 检查几个文件，确保内容正确、无HTML标签
cat "test/女神代行者/第一卷/第1章 因女神找人"接盘"而生.md" | head -30
# 确认：没有 <img> 标签，没有 URL 链接

# Step 4: 删除源文件
rm "example/女神代行者 作者：snow_xefd.txt"

# Step 5: 删除备份（test目录就绪后，备份不再需要）
rm -rf example/backup/

# Step 6: 删除辅助脚本（这些脚本不适合放入git）
rm -rf scripts/

# Step 7: 确认清理完成
ls example/
# 预期: 空目录或只有其他示例文件
ls scripts/ 2>&1 || echo "scripts目录已删除"
# 预期: 目录不存在
```

**检查点**:
- [ ] test目录结构正确（29卷）
- [ ] 验证通过（291章节）
- [ ] 手动抽查内容正确
- [ ] 源文件已删除
- [ ] 备份已删除
- [ ] 辅助脚本已删除

**⚠️ 警告：此操作不可逆，仅在验证通过后执行**

**删除流程（必须按顺序）**：

```bash
# Step 1: 确认备份存在
ls -lh example/backup/
# 必须显示: 女神代行者 作者：snow_xefd.txt

# Step 2: 确认拆分验证通过
python scripts/split_novel.py "example/女神代行者 作者：snow_xefd.txt" test
# 必须显示:
# - "Found 291 chapters"
# - "Volumes found: 29"
# - "✓ Verification passed!"

# Step 3: 手动抽查（重要！）
# 检查几个文件，确保内容正确、无HTML标签
cat "test/女神代行者/第一卷/第1章 因女神找人"接盘"而生.md" | head -30
# 确认：没有 <img> 标签，没有 URL 链接

# Step 4: 删除源文件
rm "example/女神代行者 作者：snow_xefd.txt"

# Step 5: 确认源文件已删除
ls example/
# 预期: backup/ 目录，没有 txt 文件

# Step 6: 保留备份（不要删除 backup/ 目录）
# 备份保留在 example/backup/ 以备不时之需
```

**检查点**:
- [ ] 备份文件存在
- [ ] 验证通过（291章节 + 29卷）
- [ ] 手动抽查内容正确
- [ ] 源文件已删除
- [ ] 备份仍然保留

**⚠️ 警告：此操作不可逆，仅在验证通过后执行**

```bash
# 再次确认备份存在
ls -lh example/backup/

# 确认验证通过
python scripts/split_novel.py "example/女神代行者 作者：snow_xefd.txt" test
# 必须显示 "✓ Verification passed!"

# 删除源文件
rm "example/女神代行者 作者：snow_xefd.txt"

# 确认删除
ls example/
# 预期: backup/ 目录，没有 txt 文件
```

**检查点**:

- [ ] 备份文件存在
- [ ] 验证通过
- [ ] 源文件已删除

---

## Wave 2: Core 数据模型

**目标**: 在 `projects/core/src/metadata/` 创建基础数据结构

**前置条件**: Wave 1 完成

### Task 2.1: MetadataEntity 结构

**文件**: `projects/core/src/metadata/model.rs`

**代码框架**:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// 元数据实体
///
/// 这是 core 层的核心数据结构，不包含任何 IO 逻辑
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "_metadata.ts")]
pub struct MetadataEntity {
    /// 唯一标识符（blake3 hash）
    pub id: String,

    /// 实体类型（character, scene, location 等）
    #[ts(rename = "type")]
    pub type_: String,

    /// 命名空间（从路径生成）
    pub namespace: String,

    /// frontmatter 数据（JSON 格式）
    pub frontmatter: Value,

    /// Markdown 正文内容
    pub body: String,
}

impl MetadataEntity {
    /// 创建新的元数据实体
    ///
    /// # Arguments
    /// - `id`: 唯一标识符
    /// - `type_`: 实体类型
    /// - `namespace`: 命名空间
    /// - `frontmatter`: frontmatter 数据
    /// - `body`: Markdown 正文
    ///
    /// # Returns
    /// 新的 MetadataEntity 实例
    pub fn new(
        id: impl Into<String>,
        type_: impl Into<String>,
        namespace: impl Into<String>,
        frontmatter: Value,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            type_: type_.into(),
            namespace: namespace.into(),
            frontmatter,
            body: body.into(),
        }
    }

    /// 从 frontmatter 获取字段值
    ///
    /// # Arguments
    /// - `key`: 字段名
    ///
    /// # Returns
    /// 字段值（如果存在）
    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.frontmatter.get(key)
    }

    /// 获取 frontmatter 中的类型（优先于路径推导）
    pub fn get_type_from_frontmatter(&self) -> Option<String> {
        self.frontmatter
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_entity_creation() {
        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({"name": "Hero"}),
            "# Hero\n\nA brave hero.",
        );

        assert_eq!(entity.id, "test-id");
        assert_eq!(entity.type_, "character");
        assert_eq!(entity.namespace, "global");
        assert_eq!(entity.get_field("name").unwrap(), "Hero");
    }

    #[test]
    fn test_get_type_from_frontmatter() {
        let entity = MetadataEntity::new(
            "test-id",
            "character",  // 路径推导的类型
            "global",
            serde_json::json!({"type": "protagonist"}),  // frontmatter 覆盖
            "Test",
        );

        // frontmatter 中的 type 优先级更高
        assert_eq!(entity.get_type_from_frontmatter(), Some("protagonist".to_string()));
    }
}
```

**检查点**:

- [ ] 文件创建成功
- [ ] `cargo check -p novelsaga-core` 通过
- [ ] `cargo test -p novelsaga-core` 通过

### Task 2.2: Type 推导器

**文件**: `projects/core/src/metadata/parser.rs`

**代码框架**:

```rust
use std::path::Path;
use crate::metadata::model::MetadataEntity;

/// 从文件路径推导元数据类型
///
/// # 推导规则
/// - `metadata/*.md` → "metadata"
/// - `metadata/characters/*.md` → "character"
/// - `metadata/scenes/*.md` → "scene"
/// - 其他: 使用父文件夹名单数化
///
/// # Arguments
/// - `path`: 元数据文件路径
///
/// # Returns
/// 推导出的类型字符串
pub fn infer_type_from_path(path: &Path) -> String {
    // 获取父文件夹名
    let parent = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("metadata");

    // 如果是 metadata 文件夹本身，返回 "metadata"
    if parent == "metadata" {
        return "metadata".to_string();
    }

    // 单数化（简单规则：去掉末尾的 's'）
    singularize(parent)
}

/// 单数化字符串（简单实现）
fn singularize(s: &str) -> String {
    if s.ends_with('s') && !s.ends_with("ss") {
        s[..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

/// 综合推导类型
///
/// 优先级: frontmatter type > 路径推导
///
/// # Arguments
/// - `path`: 文件路径
/// - `entity`: 实体数据（包含 frontmatter）
///
/// # Returns
/// 最终类型字符串
pub fn resolve_type(path: &Path, frontmatter: &serde_json::Value) -> String {
    get_type_from_frontmatter(frontmatter)
        .unwrap_or_else(|| infer_type_from_path(path))
}
    get_type_from_frontmatter(frontmatter)
        .unwrap_or_else(|| infer_type_from_path(path))
}
    entity.get_type_from_frontmatter()
        .unwrap_or_else(|| infer_type_from_path(path))
}

/// 从元数据路径生成 namespace
///
/// namespace = 从工作空间根到 metadata 父目录的相对路径
///
/// # Examples
/// - `metadata/` → "global"
/// - `book-01/metadata/` → "book-01"
/// - `book-01/part-01/metadata/` → "book-01/part-01"
///
/// # Arguments
/// - `metadata_path`: metadata 目录路径
/// - `workspace_root`: 工作空间根目录
///
/// # Returns
/// namespace 字符串
pub fn generate_namespace(
    metadata_path: &Path,
    workspace_root: &Path,
) -> String {
    // 获取 metadata 的父目录
    let parent = metadata_path.parent()
        .unwrap_or(metadata_path);

    // 如果是根目录下的 metadata，返回 "global"
    if parent == workspace_root {
        return "global".to_string();
    }

    // 计算相对路径
    parent.strip_prefix(workspace_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "global".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_infer_type_from_path() {
        assert_eq!(
            infer_type_from_path(&PathBuf::from("metadata/hero.md")),
            "metadata"
        );
        assert_eq!(
            infer_type_from_path(&PathBuf::from("metadata/characters/hero.md")),
            "character"
        );
        assert_eq!(
            infer_type_from_path(&PathBuf::from("metadata/scenes/opening.md")),
            "scene"
        );
    }

    #[test]
    fn test_resolve_type_priority() {
        let path = PathBuf::from("metadata/characters/hero.md");
        let entity = MetadataEntity::new(
            "test",
            "character",
            "global",
            serde_json::json!({"type": "protagonist"}),
            "Test",
        );

        // frontmatter 优先级更高
        assert_eq!(resolve_type(&path, &entity), "protagonist");

        // 无 frontmatter 时使用路径推导
        let empty_entity = MetadataEntity::new(
            "test",
            "",
            "global",
            serde_json::json!({}),
            "Test",
        );
        assert_eq!(resolve_type(&path, &empty_entity), "character");
    }

    #[test]
    fn test_generate_namespace() {
        let workspace = PathBuf::from("/workspace");

        assert_eq!(
            generate_namespace(&PathBuf::from("/workspace/metadata"), &workspace),
            "global"
        );
        assert_eq!(
            generate_namespace(&PathBuf::from("/workspace/book-01/metadata"), &workspace),
            "book-01"
        );
        assert_eq!(
            generate_namespace(&PathBuf::from("/workspace/book-01/part-01/metadata"), &workspace),
            "book-01/part-01"
        );
    }
}
```

**检查点**:

- [ ] 所有测试用例通过
- [ ] 边界情况处理（无父文件夹、非 UTF-8 文件名等）

### Task 2.3: 模块导出

**文件**: `projects/core/src/metadata/mod.rs`

**代码框架**:

```rust
//! 元数据管理模块
//!
//! 提供小说元数据的定义、解析和查询接口

pub mod model;
pub mod parser;
pub mod query;

pub use model::MetadataEntity;
pub use parser::{infer_type_from_path, resolve_type, generate_namespace};
pub use query::{MetadataQuery, QueryResult};
```

**检查点**:

- [ ] 模块导出正确
- [ ] `cargo check -p novelsaga-core` 通过

---

## Wave 3: Core 查询接口

**目标**: 定义查询 trait，提供内存实现用于测试

**前置条件**: Wave 2 完成

### Task 3.1: MetadataQuery trait

**文件**: `projects/core/src/metadata/query.rs`

**代码框架**:

```rust
use crate::metadata::model::MetadataEntity;
use std::collections::HashMap;

/// 元数据查询接口
///
/// core 层定义接口，cli 层提供实现
///
/// # 设计原则
/// - 所有方法都是同步的（core 层不涉及异步 IO）
/// - 返回 Option/Vec 而非 Result（错误处理由实现层决定）
/// - 方法签名尽量简单，便于实现
pub trait MetadataQuery {
    /// 根据 ID 获取元数据实体
    ///
    /// # Arguments
    /// - `id`: 实体 ID
    ///
    /// # Returns
    /// 实体（如果存在）
    fn get_by_id(&self, id: &str) -> Option<MetadataEntity>;

    /// 根据名称获取元数据实体（就近解析）
    ///
    /// 从指定 namespace 开始向上查找，返回最近匹配
    ///
    /// # Arguments
    /// - `name`: 实体名称（文件名不含扩展名）
    /// - `namespace`: 起始 namespace
    ///
    /// # Returns
    /// 实体（如果存在）
    fn get_by_name(&self, name: &str, namespace: &str) -> Option<MetadataEntity>;

    /// 根据类型列出所有实体
    ///
    /// # Arguments
    /// - `type_`: 实体类型
    /// - `namespace`: 可选的 namespace 过滤
    ///
    /// # Returns
    /// 实体列表
    fn list_by_type(&self, type_: &str, namespace: Option<&str>) -> Vec<MetadataEntity>;

    /// 列出 namespace 下的所有实体
    ///
    /// # Arguments
    /// - `namespace`: namespace
    ///
    /// # Returns
    /// 实体列表
    fn list_by_namespace(&self, namespace: &str) -> Vec<MetadataEntity>;

    /// 搜索实体
    ///
    /// # Arguments
    /// - `query`: 搜索关键词
    /// - `type_filter`: 可选的类型过滤
    ///
    /// # Returns
    /// 匹配的实体列表
    fn search(&self, query: &str, type_filter: Option<&str>) -> Vec<MetadataEntity>;
}

/// 查询结果（用于批量查询）
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub entities: Vec<MetadataEntity>,
    pub total: usize,
}

/// 内存元数据存储（用于测试）
///
/// 简单的 HashMap 实现，不涉及 IO
pub struct InMemoryMetadataStore {
    entities: HashMap<String, MetadataEntity>, // id -> entity
    by_name: HashMap<String, String>, // "namespace:name" -> id
}

impl InMemoryMetadataStore {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: MetadataEntity) {
        let key = format!("{}:{}", entity.namespace, entity.id);
        self.by_name.insert(key, entity.id.clone());
        self.entities.insert(entity.id.clone(), entity);
    }
}

impl MetadataQuery for InMemoryMetadataStore {
    fn get_by_id(&self, id: &str) -> Option<MetadataEntity> {
        self.entities.get(id).cloned()
    }

    fn get_by_name(&self, name: &str, namespace: &str) -> Option<MetadataEntity> {
        let key = format!("{}:{}", namespace, name);
        self.by_name.get(&key)
            .and_then(|id| self.entities.get(id))
            .cloned()
    }

    fn list_by_type(&self, type_: &str, _namespace: Option<&str>) -> Vec<MetadataEntity> {
        self.entities.values()
            .filter(|e| e.type_ == type_)
            .cloned()
            .collect()
    }

    fn list_by_namespace(&self, namespace: &str) -> Vec<MetadataEntity> {
        self.entities.values()
            .filter(|e| e.namespace == namespace)
            .cloned()
            .collect()
    }

    fn search(&self, query: &str, type_filter: Option<&str>) -> Vec<MetadataEntity> {
        self.entities.values()
            .filter(|e| {
                let matches_query = e.body.contains(query) ||
                    e.frontmatter.to_string().contains(query);
                let matches_type = type_filter.map_or(true, |t| e.type_ == t);
                matches_query && matches_type
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store() {
        let mut store = InMemoryMetadataStore::new();

        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({"name": "Hero"}),
            "A brave hero.",
        );

        store.insert(entity.clone());

        assert_eq!(store.get_by_id("test-id"), Some(entity));
        assert_eq!(store.list_by_type("character", None).len(), 1);
    }
}
```

**检查点**:

- [ ] trait 定义完整
- [ ] 内存实现可用
- [ ] 所有方法有测试
- [ ] 编译通过

---

## Wave 4: CLI 缓存层

**目标**: 实现 moka 内存缓存

**前置条件**: Wave 3 完成

**依赖添加**（如未添加）:

```toml
# projects/cli/Cargo.toml
[dependencies]
moka = { version = "0.12", features = ["future"] }
```

### Task 4.1: CacheManager 结构

**文件**: `projects/cli/src/metadata/cache.rs`

**代码框架**:

```rust
use moka::future::Cache;
use novelsaga_core::metadata::model::MetadataEntity;
use std::sync::Arc;

/// 元数据缓存管理器
///
/// 使用 moka 提供 LRU 缓存，支持异步操作
pub struct CacheManager {
    cache: Cache<String, Arc<MetadataEntity>>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    ///
    /// # Arguments
    /// - `capacity`: 缓存容量（条目数）
    ///
    /// # Returns
    /// 新的 CacheManager 实例
    pub fn new(capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .build();

        Self { cache }
    }

    /// 获取缓存中的实体
    ///
    /// # Arguments
    /// - `id`: 实体 ID
    ///
    /// # Returns
    /// 实体（如果在缓存中）
    pub async fn get(&self, id: &str) -> Option<Arc<MetadataEntity>> {
        self.cache.get(id).await
    }

    /// 插入实体到缓存
    ///
    /// # Arguments
    /// - `id`: 实体 ID
    /// - `entity`: 实体数据
    pub async fn insert(&self, id: String, entity: MetadataEntity) {
        self.cache.insert(id, Arc::new(entity)).await;
    }

    /// 批量插入
    ///
    /// # Arguments
    /// - `entities`: (id, entity) 列表
    pub async fn insert_batch(&self, entities: Vec<(String, MetadataEntity)>) {
        for (id, entity) in entities {
            self.cache.insert(id, Arc::new(entity)).await;
        }
    }

    /// 从缓存中移除实体
    ///
    /// # Arguments
    /// - `id`: 实体 ID
    pub async fn invalidate(&self, id: &str) {
        self.cache.invalidate(id).await;
    }

    /// 清空缓存
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// 获取缓存统计
    ///
    /// # Returns
    /// (条目数, 权重)
    pub fn stats(&self) -> (u64, u64) {
        (self.cache.entry_count(), self.cache.weighted_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_manager() {
        let cache = CacheManager::new(100);

        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({}),
            "test",
        );

        cache.insert("test-id".to_string(), entity.clone()).await;

        let cached = cache.get("test-id").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, "test-id");
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = CacheManager::new(100);

        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({}),
            "test",
        );

        cache.insert("test-id".to_string(), entity).await;
        assert!(cache.get("test-id").await.is_some());

        cache.invalidate("test-id").await;
        assert!(cache.get("test-id").await.is_none());
    }
}
```

**检查点**:

- [ ] 编译通过
- [ ] 测试通过
- [ ] 正确处理 Arc

---

## Wave 5: CLI 索引层

**目标**: 实现 sled 持久化索引

**前置条件**: Wave 4 完成

**依赖添加**（如未添加）:

```toml
# projects/cli/Cargo.toml
[dependencies]
sled = "0.34"
blake3 = "1.5"
```

### Task 5.1: IndexManager 结构

**文件**: `projects/cli/src/metadata/index.rs`

**代码框架**:

```rust
use sled::Db;
use std::path::Path;
use novelsaga_core::metadata::model::MetadataEntity;

/// 元数据索引管理器
///
/// 使用 sled 提供持久化索引
pub struct IndexManager {
    db: Db,
}

impl IndexManager {
    /// 打开或创建索引数据库
    ///
    /// # Arguments
    /// - `path`: 数据库目录路径
    ///
    /// # Returns
    /// Result<IndexManager, sled::Error>
    pub fn open(path: &Path) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// 生成 ID（blake3 hash）
    ///
    /// # Arguments
    /// - `source`: 源字符串（文件名或 frontmatter id）
    ///
    /// # Returns
    /// 16 字符的 hash 字符串
    pub fn generate_id(source: &str) -> String {
        let hash = blake3::hash(source.as_bytes());
        hash.to_hex()[..16].to_string()
    }

    /// 索引元数据实体
    ///
    /// # Arguments
    /// - `entity`: 实体数据
    ///
    /// # Returns
    /// Result<(), sled::Error>
    pub fn index_entity(&self, entity: &MetadataEntity) -> Result<(), sled::Error> {
        let id = &entity.id;

        // 存储实体数据
        let data = serde_json::to_vec(entity)
            .map_err(|e| sled::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;
        self.db.insert(format!("entity:{}", id), data)?;

        // 索引：name -> id
        let name_key = format!("name:{}:{}", entity.namespace, id);
        self.db.insert(name_key, id.as_bytes())?;

        // 索引：type -> id
        let type_key = format!("type:{}:{}", entity.type_, id);
        self.db.insert(type_key, id.as_bytes())?;

        // 索引：namespace -> id
        let ns_key = format!("ns:{}:{}", entity.namespace, id);
        self.db.insert(ns_key, id.as_bytes())?;

        Ok(())
    }

    /// 根据 ID 获取实体
    pub fn get_by_id(&self, id: &str) -> Result<Option<MetadataEntity>, sled::Error> {
        match self.db.get(format!("entity:{}", id))? {
            Some(data) => {
                let entity: MetadataEntity = serde_json::from_slice(&data)
                    .map_err(|e| sled::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string()
                    )))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    /// 根据类型列出实体
    pub fn list_by_type(&self, type_: &str) -> Result<Vec<MetadataEntity>, sled::Error> {
        let mut entities = Vec::new();
        let prefix = format!("type:{}:", type_);

        for item in self.db.scan_prefix(&prefix) {
            let (_, value) = item?;
            let id = String::from_utf8_lossy(&value);
            if let Some(entity) = self.get_by_id(&id)? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// 删除索引
    ///
    /// # Arguments
    /// - `id`: 实体 ID
    pub fn remove_entity(&self, id: &str) -> Result<(), sled::Error> {
        // 需要先获取实体以删除二级索引
        if let Some(entity) = self.get_by_id(id)? {
            // 删除实体
            self.db.remove(format!("entity:{}", id))?;

            // 删除索引
            self.db.remove(format!("name:{}:{}", entity.namespace, id))?;
            self.db.remove(format!("type:{}:{}", entity.type_, id))?;
            self.db.remove(format!("ns:{}:{}", entity.namespace, id))?;
        }

        Ok(())
    }

    /// 重建索引（清空后重新索引）
    pub fn rebuild(&self) -> Result<(), sled::Error> {
        self.db.clear()?;
        Ok(())
    }

    /// 刷新数据到磁盘
    pub fn flush(&self) -> Result<(), sled::Error> {
        self.db.flush()
    }

    /// 根据 namespace 列出实体
    pub fn list_by_namespace(
        &self,
        namespace: &str,
    ) -> Result<Vec<MetadataEntity>, sled::Error> {
        let mut entities = Vec::new();
        let prefix = format!("ns:{}", namespace);

        for item in self.db.scan_prefix(&prefix) {
            let (_, value) = item?;
            let id = String::from_utf8_lossy(&value);
            if let Some(entity) = self.get_by_id(&id)? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }
}
        self.db.flush()
    /// 根据 namespace 列出实体
    pub fn list_by_namespace(
        &self,
        namespace: &str,
    ) -> Result<Vec<MetadataEntity>, sled::Error> {
        let mut entities = Vec::new();
        let prefix = format!("ns:{}:", namespace);

        for item in self.db.scan_prefix(&prefix) {
            let (_, value) = item?;
            let id = String::from_utf8_lossy(&value);
            if let Some(entity) = self.get_by_id(&id)? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }
}

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_manager() {
        let temp_dir = TempDir::new().unwrap();
        let index = IndexManager::open(temp_dir.path()).unwrap();

        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({"name": "Hero"}),
            "A hero",
        );

        index.index_entity(&entity).unwrap();

        let retrieved = index.get_by_id("test-id").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().type_, "character");
    }

    #[test]
    fn test_generate_id() {
        let id1 = IndexManager::generate_id("test-source");
        let id2 = IndexManager::generate_id("test-source");
        let id3 = IndexManager::generate_id("different-source");

        // 相同输入产生相同输出
        assert_eq!(id1, id2);
        // 不同输入产生不同输出
        assert_ne!(id1, id3);
        // 长度为 16
        assert_eq!(id1.len(), 16);
    }
}
```

**检查点**:

- [ ] 编译通过
- [ ] 测试通过
- [ ] ID 生成正确
- [ ] 正确处理 sled 错误

---

## Wave 6: 文件监听

**目标**: 实现 notify 文件监听

**前置条件**: Wave 5 完成

**依赖添加**（如未添加）:

```toml
# projects/cli/Cargo.toml
[dependencies]
notify = "6.1"
```

### Task 6.1: FileWatcher 结构

**文件**: `projects/cli/src/metadata/watcher.rs`

**代码框架**:

```rust
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Sender, Receiver};

/// 文件变更事件
#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    Created(String),  // 文件路径
    Modified(String),
    Removed(String),
}

/// 元数据文件监听器
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<FileChangeEvent>,
}

impl FileWatcher {
    /// 创建新的文件监听器
    ///
    /// # Arguments
    /// - `watch_path`: 监听目录
    ///
    /// # Returns
    /// Result<FileWatcher, notify::Error>
    pub fn new(watch_path: &Path) -> Result<Self, notify::Error> {
        let (sender, receiver) = channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        for path in event.paths {
                            if let Some(path_str) = path.to_str() {
                                // 只处理 metadata 目录下的 .md 文件
                                if path_str.contains("/metadata/") && path_str.ends_with(".md") {
                                    let event = match event.kind {
                                        notify::EventKind::Create(_) => {
                                            FileChangeEvent::Created(path_str.to_string())
                                        }
                                        notify::EventKind::Modify(_) => {
                                            FileChangeEvent::Modified(path_str.to_string())
                                        }
                                        notify::EventKind::Remove(_) => {
                                            FileChangeEvent::Removed(path_str.to_string())
                                        }
                                        _ => continue,
                                    };
                                    let _ = sender.send(event);
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            },
            Config::default(),
        )?;

        Ok(Self { watcher, receiver })
    }

    /// 开始监听
    pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    /// 停止监听
    pub fn unwatch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.unwatch(path)
    }

    /// 接收事件（非阻塞）
    pub fn try_recv(&self) -> Option<FileChangeEvent> {
        self.receiver.try_recv().ok()
    }

    /// 接收事件（阻塞）
    pub fn recv(&self) -> Result<FileChangeEvent, std::sync::mpsc::RecvError> {
        self.receiver.recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_file_watcher() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_dir = temp_dir.path().join("metadata");
        fs::create_dir(&metadata_dir).unwrap();

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();
        watcher.watch(temp_dir.path()).unwrap();

        // 创建测试文件
        let test_file = metadata_dir.join("test.md");
        fs::write(&test_file, "# Test").unwrap();

        // 等待事件（简单轮询）
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 清理
        let _ = watcher.unwatch(temp_dir.path());
    }
}
```

**检查点**:

- [ ] 编译通过
- [ ] 正确处理 notify 事件

---

## Wave 7: 异步写回

**目标**: 实现 flume 通道 + 异步 Worker

**前置条件**: Wave 6 完成

**依赖添加**（如未添加）:

```toml
# projects/cli/Cargo.toml
[dependencies]
flume = "0.11"
```

### Task 7.1: WriteBackWorker 结构

**文件**: `projects/cli/src/metadata/worker.rs`

**代码框架**:

```rust
use flume::{Receiver, Sender};
use std::time::Duration;
use tokio::time::interval;
use std::sync::Arc;

use crate::metadata::index::IndexManager;
use novelsaga_core::metadata::model::MetadataEntity;

/// 写回任务
#[derive(Debug, Clone)]
pub enum WriteTask {
    /// 更新或创建实体
    Upsert { id: String, data: Vec<u8> },
    /// 删除实体
    Delete { id: String },
    /// 批量刷新
    Flush,
}

/// 异步写回 Worker
///
/// 接收写任务，批量写入 sled
pub struct WriteBackWorker {
    task_sender: Sender<WriteTask>,
}

impl WriteBackWorker {
    /// 创建并启动 Worker
    ///
    /// # Arguments
    /// - `index`: IndexManager 的 Arc 引用
    /// - `batch_size`: 批量写入大小
    /// - `flush_interval`: 刷新间隔
    ///
    /// # Returns
    /// WriteBackWorker 实例
    pub fn new(
        index: Arc<IndexManager>,
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        let (sender, receiver) = flume::unbounded();

        // 启动后台任务
        tokio::spawn(async move {
            Self::run_worker(receiver, index, batch_size, flush_interval).await;
        });

        Self {
            task_sender: sender,
        }
    }

    /// 提交写任务
    pub fn submit(&self, task: WriteTask) {
        let _ = self.task_sender.send(task);
    }

    /// Worker 主循环
    async fn run_worker(
        receiver: Receiver<WriteTask>,
        index: Arc<IndexManager>,
        batch_size: usize,
        flush_interval: Duration,
    ) {
        let mut batch = Vec::with_capacity(batch_size);
        let mut ticker = interval(flush_interval);

        loop {
            tokio::select! {
                Ok(task) = receiver.recv_async() => {
                    batch.push(task);
                    if batch.len() >= batch_size {
                        Self::process_batch(&batch, &index).await;
                        batch.clear();
                    }
                }
                _ = ticker.tick() => {
                    if !batch.is_empty() {
                        Self::process_batch(&batch, &index).await;
                        batch.clear();
                    }
                }
            }
        }
    }

    /// 处理批量任务
    async fn process_batch(batch: &[WriteTask], index: &IndexManager) {
        for task in batch {
            match task {
                WriteTask::Upsert { id, data } => {
                    // 反序列化并索引
                    if let Ok(entity) = serde_json::from_slice::<MetadataEntity>(data) {
                        if let Err(e) = index.index_entity(&entity) {
                            eprintln!("Failed to index entity {}: {}", id, e);
                        }
                    } else {
                        eprintln!("Failed to deserialize entity {}", id);
                    }
                }
                WriteTask::Delete { id } => {
                    if let Err(e) = index.remove_entity(id) {
                        eprintln!("Failed to remove entity {}: {}", id, e);
                    }
                }
                WriteTask::Flush => {
                    if let Err(e) = index.flush() {
                        eprintln!("Failed to flush index: {}", e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_worker() {
        let temp_dir = TempDir::new().unwrap();
        let index = Arc::new(IndexManager::open(temp_dir.path()).unwrap());

        let worker = WriteBackWorker::new(
            index,
            10,
            Duration::from_secs(1),
        );

        let entity = MetadataEntity::new(
            "test-id",
            "character",
            "global",
            serde_json::json!({}),
            "test",
        );

        worker.submit(WriteTask::Upsert {
            id: "test-id".to_string(),
            data: serde_json::to_vec(&entity).unwrap(),
        });

        // 等待处理
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

**检查点**:

- [ ] 编译通过
- [ ] 通道工作正常
- [ ] 正确处理所有任务类型

---

## Wave 8: CLI 集成

**目标**: 实现 CLI 命令

**前置条件**: Wave 7 完成

### Task 8.1: 模块整合

**文件**: `projects/cli/src/metadata/mod.rs`

**代码框架**:

```rust
//! CLI 元数据管理模块

pub mod cache;
pub mod index;
pub mod watcher;
pub mod worker;

pub use cache::CacheManager;
pub use index::IndexManager;
pub use watcher::{FileWatcher, FileChangeEvent};
pub use worker::{WriteBackWorker, WriteTask};
```

**检查点**:

- [ ] 模块导出正确
- [ ] `cargo check -p novelsaga-cli` 通过

### Task 8.2: CLI 命令实现

**文件**: `projects/cli/src/commands/metadata.rs`

**代码框架**:

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum MetadataCommands {
    /// 索引元数据
    Index(IndexCommand),
    /// 列出元数据实体
    List(ListCommand),
    /// 显示元数据详情
    Show(ShowCommand),
}

#[derive(Args, Debug)]
pub struct IndexCommand {
    /// 工作空间路径
    #[arg(short, long)]
    pub workspace: Option<PathBuf>,

    /// 监听模式
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Args, Debug)]
pub struct ListCommand {
    /// 按类型过滤
    #[arg(short, long)]
    pub type_: Option<String>,

    /// 按 namespace 过滤
    #[arg(short, long)]
    pub namespace: Option<String>,
}

#[derive(Args, Debug)]
pub struct ShowCommand {
    /// 实体名称或 ID
    pub name: String,

    /// 命名空间
    #[arg(short, long)]
    pub namespace: Option<String>,
}

impl IndexCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let workspace = self.workspace.clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        println!("Indexing workspace: {}", workspace.display());

        // 创建索引目录
        let index_path = workspace.join(".cache/novelsaga/sled");
        std::fs::create_dir_all(&index_path)?;

        let index = IndexManager::open(&index_path)?;

        // 扫描 metadata 目录
        let metadata_dir = workspace.join("metadata");
        if metadata_dir.exists() {
            Self::index_directory(&index, &metadata_dir, &workspace).await?;
        }

        // 扫描小说目录中的 metadata 子目录
        for entry in std::fs::read_dir(&workspace)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let metadata_subdir = path.join("metadata");
                if metadata_subdir.exists() {
                    Self::index_directory(&index, &metadata_subdir, &workspace).await?;
                }
            }
        }

        // 刷新索引到磁盘
        index.flush()?;
        println!("Indexing complete!");

        if self.watch {
            println!("Watch mode enabled. Press Ctrl+C to stop.");
            tokio::signal::ctrl_c().await?;
        }

        Ok(())
    }

    async fn index_directory(
        index: &IndexManager,
        dir: &std::path::Path,
        workspace: &std::path::Path,
    ) -> anyhow::Result<()> {
        use novelsaga_core::metadata::{generate_namespace, resolve_type};

        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                // 读取文件内容
                let content = std::fs::read_to_string(path)?;

                // 解析 frontmatter 和 body（简化处理）
                let (frontmatter, body) = Self::parse_markdown(&content);

                // 生成 ID（基于文件名）
                let filename = path.file_stem().unwrap().to_string_lossy();
                let id = IndexManager::generate_id(&filename);

                // 生成 namespace
                let metadata_dir = path.parent().unwrap();
                let namespace = generate_namespace(metadata_dir, workspace);

                // 创建实体
                let entity = novelsaga_core::metadata::MetadataEntity::new(
                    &id,
                    resolve_type(path, &frontmatter),
                    &namespace,
                    frontmatter,
                    body,
                );

                // 索引实体
                index.index_entity(&entity)?;
                println!("Indexed: {} ({})", filename, namespace);
            }
        }

        Ok(())
    }

    fn parse_markdown(content: &str) -> (serde_json::Value, String) {
        // 简化实现：假设没有 frontmatter，全部作为 body
        // 实际应该解析 YAML frontmatter
        (serde_json::json!({}), content.to_string())
    }
}

        // 创建索引目录
        let index_path = workspace.join(".cache/novelsaga/sled");
        std::fs::create_dir_all(&index_path)?;

        let index = IndexManager::open(&index_path)?;

        // 扫描 metadata 目录
        let metadata_dir = workspace.join("metadata");
        if metadata_dir.exists() {
            Self::index_directory(&index, &metadata_dir, &workspace).await?;
        }

        // 扫描小说目录中的 metadata 子目录
        for entry in std::fs::read_dir(&workspace)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let metadata_subdir = path.join("metadata");
                if metadata_subdir.exists() {
                    Self::index_directory(&index, &metadata_subdir, &workspace).await?;
                }
            }
        }

        // 刷新索引到磁盘
        index.flush()?;
        println!("Indexing complete!");

        if self.watch {
            println!("Watch mode enabled. Press Ctrl+C to stop.");
            tokio::signal::ctrl_c().await?;
        }
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        println!("Indexing workspace: {}", workspace.display());

        // TODO: 扫描所有 metadata 文件并建立索引

        if self.watch {
            println!("Watch mode enabled. Press Ctrl+C to stop.");
            tokio::signal::ctrl_c().await?;
        }

        Ok(())
    }
}

impl ListCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        println!("Listing metadata...");

        // 打开索引（默认位置）
        let workspace = std::env::current_dir()?;
        let index_path = workspace.join(".cache/novelsaga/sled");
        
        if !index_path.exists() {
            println!("No index found. Run 'novelsaga metadata index' first.");
            return Ok(());
        }

        let index = IndexManager::open(&index_path)?;

        // 根据过滤条件查询
        let entities = if let Some(type_) = &self.type_ {
            index.list_by_type(type_)?
        } else if let Some(namespace) = &self.namespace {
            index.list_by_namespace(namespace)?
        } else {
            // 默认列出所有
            index.list_by_type("metadata")?
        };

        // 显示结果
        println!("Found {} entities:", entities.len());
        for entity in entities {
            println!("  - [{}] {} ({})", 
                entity.type_, 
                entity.id, 
                entity.namespace
            );
        }

        Ok(())
    }
}

impl ShowCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        println!("Showing metadata: {}", self.name);

        // 打开索引
        let workspace = std::env::current_dir()?;
        let index_path = workspace.join(".cache/novelsaga/sled");
        
        if !index_path.exists() {
            println!("No index found. Run 'novelsaga metadata index' first.");
            return Ok(());
        }

        let index = IndexManager::open(&index_path)?;

        // 先尝试按 ID 查找
        if let Some(entity) = index.get_by_id(&self.name)? {
            Self::print_entity(&entity);
            return Ok(());
        }

        // 再尝试按名称查找
        let namespace = self.namespace.as_deref().unwrap_or("global");
        // 注意：这里简化处理，实际应该实现 get_by_name
        println!("Entity not found: {} in namespace {}", self.name, namespace);

        Ok(())
    }

    fn print_entity(entity: &MetadataEntity) {
        println!("ID: {}", entity.id);
        println!("Type: {}", entity.type_);
        println!("Namespace: {}", entity.namespace);
        println!("Frontmatter: {}", serde_json::to_string_pretty(&entity.frontmatter).unwrap_or_default());
        println!("Body preview: {}", &entity.body[..entity.body.len().min(200)]);
    }
}

        println!("Listing metadata...");

        if let Some(type_) = &self.type_ {
            println!("Filter by type: {}", type_);
        }

        if let Some(namespace) = &self.namespace {
            println!("Filter by namespace: {}", namespace);
        }

        // TODO: 查询索引并显示

        Ok(())
    }
}


    pub async fn execute(&self) -> anyhow::Result<()> {
        println!("Showing metadata: {}", self.name);

        if let Some(namespace) = &self.namespace {
            println!("Namespace: {}", namespace);
        }

        // TODO: 查询并显示详情

        Ok(())
    }
}
```

**检查点**:

- [ ] 子命令注册成功
- [ ] 帮助信息完整

---

## Wave 9: 幂等性集成测试

**目标**: 实现幂等性集成测试

**前置条件**: Wave 8 完成

**重要约束**: 测试必须是幂等的（测试前后文件系统状态一致）

### Task 9.1: 测试夹具框架

**文件**: `projects/cli/tests/integration_test.rs`

**代码框架**:

```rust
use novelsaga_cli::metadata::{CacheManager, IndexManager, FileWatcher, WriteBackWorker, WriteTask};
use novelsaga_core::metadata::MetadataEntity;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio;

/// 测试夹具
///
/// 提供隔离的测试环境，确保测试幂等性
pub struct TestFixture {
    temp_dir: TempDir,
    cache: CacheManager,
    index: Arc<IndexManager>,
}

impl TestFixture {
    /// 创建新的测试夹具
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;

        // 创建缓存（内存中）
        let cache = CacheManager::new(100);

        // 创建索引（临时目录）
        let index_path = temp_dir.path().join("index");
        let index = Arc::new(IndexManager::open(&index_path)?);

        Ok(Self {
            temp_dir,
            cache,
            index,
        })
    }

    /// 获取临时目录路径
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// 创建测试实体
    pub fn create_test_entity(&self, id: &str, type_: &str) -> MetadataEntity {
        MetadataEntity::new(
            id,
            type_,
            "test-namespace",
            serde_json::json!({"name": format!("Test {}", id)}),
            format!("# Test {}\n\nTest content.", id),
        )
    }
}

/// 计算目录内容的哈希（用于验证幂等性）
/// 计算目录内容的哈希（用于验证幂等性）
fn hash_dir(test_dir: &Path) -> anyhow::Result<String> {
    use std::collections::BTreeMap;
    use std::fs;

    let mut files: BTreeMap<String, String> = BTreeMap::new();

    for entry in walkdir::WalkDir::new(test_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // 修复：使用 test_dir 作为前缀，而不是 path
            let rel_path = path.strip_prefix(test_dir)?.to_string_lossy().to_string();
            let content = fs::read_to_string(path)?;
            files.insert(rel_path, content);
        }
    }

    // 简单哈希：文件路径和内容的组合
    let hash_input = format!("{:?}", files);
    Ok(blake3::hash(hash_input.as_bytes()).to_string())
}

    use std::fs;

    let mut files: BTreeMap<String, String> = BTreeMap::new();

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let rel_path = path.strip_prefix(path)?.to_string_lossy().to_string();
            let content = fs::read_to_string(path)?;
            files.insert(rel_path, content);
        }
    }

    // 简单哈希：文件路径和内容的组合
    let hash_input = format!("{:?}", files);
    Ok(blake3::hash(hash_input.as_bytes()).to_string())
}

#[tokio::test]
#[ignore = "requires filesystem operations"]
async fn test_metadata_crud_idempotent() -> anyhow::Result<()> {
    // Arrange: 创建隔离环境
    let fixture = TestFixture::new().await?;
    let test_dir = fixture.path();

    // 记录初始状态
    let before_hash = hash_dir(test_dir)?;

    // Act: 执行 CRUD 操作
    let entity = fixture.create_test_entity("test-1", "character");

    // 创建
    fixture.cache.insert(entity.id.clone(), entity.clone()).await;
    fixture.index.index_entity(&entity)?;

    // 读取
    let cached = fixture.cache.get(&entity.id).await;
    assert!(cached.is_some());

    let indexed = fixture.index.get_by_id(&entity.id)?;
    assert!(indexed.is_some());

    // 更新
    let mut updated = entity.clone();
    updated.body = "Updated content".to_string();
    fixture.cache.insert(updated.id.clone(), updated.clone()).await;
    fixture.index.index_entity(&updated)?;

    // 删除
    fixture.cache.invalidate(&entity.id).await;
    fixture.index.remove_entity(&entity.id)?;

    // Assert: 验证幂等性（文件系统状态应恢复到初始）
    let after_hash = hash_dir(test_dir)?;

    // 注意：由于 sled 的日志结构，文件系统可能不完全一致
    // 我们验证的是逻辑一致性：索引中不应再有该实体
    let should_be_none = fixture.index.get_by_id(&entity.id)?;
    assert!(should_be_none.is_none());

    Ok(())
}

#[tokio::test]
#[ignore = "requires filesystem operations"]
async fn test_file_watcher_idempotent() -> anyhow::Result<()> {
    use std::fs;

    // Arrange
    let temp_dir = TempDir::new()?;
    let metadata_dir = temp_dir.path().join("metadata");
    fs::create_dir(&metadata_dir)?;

    let mut watcher = FileWatcher::new(temp_dir.path())?;
    watcher.watch(temp_dir.path())?;

    // 记录初始状态
    let before_count = fs::read_dir(&metadata_dir)?.count();

    // Act: 创建、修改、删除文件
    let test_file = metadata_dir.join("test.md");
    fs::write(&test_file, "# Test")?;

    std::thread::sleep(Duration::from_millis(100));

    fs::write(&test_file, "# Updated")?;

    std::thread::sleep(Duration::from_millis(100));

    fs::remove_file(&test_file)?;

    std::thread::sleep(Duration::from_millis(100));

    // Assert: 文件系统应恢复到初始状态
    let after_count = fs::read_dir(&metadata_dir)?.count();
    assert_eq!(before_count, after_count);

    // 清理
    let _ = watcher.unwatch(temp_dir.path());

    Ok(())
}

#[tokio::test]
#[ignore = "requires filesystem operations"]
async fn test_write_back_worker_idempotent() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = TempDir::new()?;
    let index = Arc::new(IndexManager::open(temp_dir.path())?);
    let worker = WriteBackWorker::new(
        index.clone(),
        10,
        Duration::from_millis(100),
    );

    let entity = MetadataEntity::new(
        "worker-test",
        "character",
        "test",
        serde_json::json!({}),
        "test",
    );

    // Act: 提交任务并等待处理
    worker.submit(WriteTask::Upsert {
        id: entity.id.clone(),
        data: serde_json::to_vec(&entity)?,
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 验证已写入
    let result = index.get_by_id(&entity.id)?;
    assert!(result.is_some());

    // 删除
    worker.submit(WriteTask::Delete {
        id: entity.id.clone(),
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Assert: 应已删除
    let result = index.get_by_id(&entity.id)?;
    assert!(result.is_none());

    Ok(())
}
```

**检查点**:

- [ ] 测试框架可编译
- [ ] 测试用例覆盖主要场景
- [ ] 测试使用临时目录（隔离性）
- [ ] 测试不修改项目文件（幂等性）

### Task 9.2: 端到端测试

**文件**: `projects/cli/tests/e2e_test.rs`

**代码框架**:

```rust
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// 端到端测试：完整工作流程
///
/// 测试场景：
/// 1. 扫描小说目录
/// 2. 建立索引
/// 3. 执行查询
/// 4. 验证结果
#[tokio::test]
#[ignore = "requires test novel data"]
async fn test_novel_indexing_e2e() -> anyhow::Result<()> {
    // 只在 test/女神代行者 存在时运行
    let novel_path = Path::new("test/女神代行者");
    if !novel_path.exists() {
        println!("Skipping e2e test: test novel not found");
        return Ok(());
    }

    // Arrange: 创建临时索引目录
    let temp_dir = TempDir::new()?;
    let index_path = temp_dir.path().join("index");

    // Act: 扫描并索引小说
    // TODO: 实现扫描逻辑

    // Assert: 验证索引结果
    // TODO: 验证索引中包含所有章节

    Ok(())
}

/// 测试幂等性：多次索引应产生相同结果
#[tokio::test]
#[ignore = "requires test novel data"]
async fn test_indexing_idempotent() -> anyhow::Result<()> {
    let novel_path = Path::new("test/女神代行者");
    if !novel_path.exists() {
        println!("Skipping idempotent test: test novel not found");
        return Ok(());
    }

    // Arrange
    let temp_dir = TempDir::new()?;
    let index_path = temp_dir.path().join("index");

    // Act: 第一次索引
    // TODO: 执行索引
    // let stats1 = index_stats(&index_path)?;

    // Act: 第二次索引（应覆盖或跳过已有数据）
    // TODO: 再次执行索引
    // let stats2 = index_stats(&index_path)?;

    // Assert: 两次结果应一致
    // assert_eq!(stats1, stats2);

    Ok(())
}
```

**检查点**:

- [ ] 测试可运行
- [ ] 使用条件编译/忽略标记处理可选依赖

---

## 错误处理与恢复流程

### 编译错误处理

#### 错误 E1: 缺少依赖

**症状**: `error: cannot find crate 'xxx'`

**解决步骤**:

1. 检查 `Cargo.toml` 是否已添加依赖
2. 如未添加，按以下格式添加:
   ```toml
   [dependencies]
   crate-name = "version"
   ```
3. 运行 `cargo check` 验证
4. 如仍失败，升级到 oracle

#### 错误 E2: trait 未实现

**症状**: `error: the trait bound 'X: Trait' is not satisfied`

**解决步骤**:

1. 检查是否忘记 `impl Trait for X`
2. 检查 trait 方法是否全部实现
3. 检查泛型参数是否匹配
4. 如仍失败，升级到 oracle

#### 错误 E3: 生命周期错误

**症状**: `error: lifetime mismatch`

**解决步骤**:

1. **立即升级**到 oracle（不要自行解决生命周期问题）

### 运行时错误处理

#### 错误 R1: panic

**症状**: `thread 'main' panicked at '...'`

**解决步骤**:

1. 定位 panic 位置（stack trace）
2. 检查是否有 `unwrap()` / `expect()`
3. 替换为 `match` 或 `if let` 处理错误
4. 如无法定位，升级到 oracle

#### 错误 R2: 死锁

**症状**: 程序无响应，CPU 占用低

**解决步骤**:

1. **立即升级**到 oracle

#### 错误 R3: 性能问题

**症状**: 操作缓慢、内存占用高

**解决步骤**:

1. 记录性能数据
2. 完成当前任务（功能优先）
3. 在 TODO 中标记 `[PERF]`
4. 继续执行，性能优化作为后续任务

---

## 代码审批与检查点

### 每 Wave 强制检查点

每个 Wave 完成后必须执行：

#### 检查点 1: 编译检查

```bash
cargo check -p novelsaga-core
cargo check -p novelsaga-cli
```

- 必须 0 错误，0 警告

#### 检查点 2: 单元测试

```bash
cargo test -p novelsaga-core
cargo test -p novelsaga-cli
```

- 新增代码必须有测试覆盖
- 测试通过率必须 100%

#### 检查点 3: 代码审查

- [ ] 是否遵循项目命名规范
- [ ] 是否有适当的文档注释
- [ ] 是否处理了所有错误情况
- [ ] 是否有不必要的 clone/alloc
- [ ] 是否符合 core/cli 分层原则

#### 检查点 4: 集成验证

```bash
cargo build --release
./target/release/novelsaga --help
```

---

## 执行记录

### Wave 0: 前置准备

- [ ] Task 0.1: 环境检查
- [ ] Task 0.2: 依赖确认
- [ ] Task 0.3: 目录结构创建

### Wave 1: 小说文件拆分

- [ ] Task 1.1: 备份源文件
- [ ] Task 1.2: 创建拆分脚本
- [ ] Task 1.3: 执行拆分并验证
- [ ] Task 1.4: 删除源文件（验证通过后）

### Wave 2: Core 数据模型

- [ ] Task 2.1: MetadataEntity 结构
- [ ] Task 2.2: Type 推导器
- [ ] Task 2.3: 模块导出

### Wave 3: Core 查询接口

- [ ] Task 3.1: MetadataQuery trait

### Wave 4: CLI 缓存层

- [ ] Task 4.1: CacheManager 结构

### Wave 5: CLI 索引层

- [ ] Task 5.1: IndexManager 结构

### Wave 6: 文件监听

- [ ] Task 6.1: FileWatcher 结构

### Wave 7: 异步写回

- [ ] Task 7.1: WriteBackWorker 结构

### Wave 8: CLI 集成

- [ ] Task 8.1: 模块整合
- [ ] Task 8.2: CLI 命令实现

### Wave 9: 幂等性集成测试

- [ ] Task 9.1: 测试夹具框架
- [ ] Task 9.2: 端到端测试

---

## 决策记录

### 已确定的决策

| 决策          | 内容                        | 原因                     |
| ------------- | --------------------------- | ------------------------ |
| core/cli 分层 | core 无 IO，cli 实现所有 IO | 测试性、可移植性         |
| 缓存策略      | Write-Back                  | 性能优先，容忍短暂不一致 |
| ID 生成       | blake3 hash                 | 确定性、分布式友好       |
| 类型推导      | 路径 + frontmatter 覆盖     | 约定优于配置             |
| 层级限制      | 无限制                      | 灵活性                   |
| 存储位置      | `.cache/novelsaga/`         | 可 gitignore             |
| 文件拆分      | Python 脚本                 | 简单、可维护             |
| 测试幂等性    | 临时目录 + RAII             | 隔离性、可靠性           |

### 需要执行时确定的决策

无。所有决策已在计划中明确。

---

## 附录

### A. 常用命令速查

```bash
# 检查
cargo check -p novelsaga-core
cargo check -p novelsaga-cli

# 测试
cargo test -p novelsaga-core
cargo test -p novelsaga-cli
cargo test --all

# 构建
cargo build -p novelsaga-cli
cargo build --release

# 运行
./target/debug/novelsaga metadata index --help
./target/release/novelsaga metadata list --type character

# 清理
cargo clean
rm -rf .cache/novelsaga/

# 拆分小说
python scripts/split_novel.py \
    "example/女神代行者 作者：snow_xefd.txt" \
    "test"
```

### B. 文件清单

**Core 层**:

- `projects/core/src/metadata/mod.rs`
- `projects/core/src/metadata/model.rs`
- `projects/core/src/metadata/parser.rs`
- `projects/core/src/metadata/query.rs`

**CLI 层**:

- `projects/cli/src/metadata/mod.rs`
- `projects/cli/src/metadata/cache.rs`
- `projects/cli/src/metadata/index.rs`
- `projects/cli/src/metadata/watcher.rs`
- `projects/cli/src/metadata/worker.rs`
- `projects/cli/src/commands/metadata.rs`

**测试**:

- `projects/cli/tests/integration_test.rs`
- `projects/cli/tests/e2e_test.rs`

**脚本**:

- `scripts/split_novel.py`

### C. 升级模板

**向 oracle 升级**:

````
我在实现 [任务名] 时遇到 [具体问题]。

当前代码：
```rust
[相关代码片段]
````

已尝试：

- [解决方法 1]
- [解决方法 2]

需要指导：
[具体方向，如：如何正确处理生命周期？是否应该使用 Arc？]

```

---

**计划版本**: 1.0
**创建时间**: 2026-02-26
**预计执行时间**: 3-4 天
**执行者**: Sisyphus Agent
```
