# NovelSaga CLI - 下一步工作计划

**更新时间**: 2026-02-03
**当前状态**: ✅ 代码质量清理完成，准备进入核心功能开发

---

## 📋 下次启动时的 TODO

### 立即开始 (P1)

- [ ] **错误消息优化** - 改进用户友好的错误提示
  - 文件: `projects/cli/src/bridge/error.rs`, `assets/error.rs`
  - 目标: 从 "IO Error" → "未找到 Node.js，请使用 --node-path 指定路径"

### 接下来 (P2)

- [ ] **元数据管理器** - 小说元数据模型和存储
  - 核心数据结构设计
  - 内存存储和查询
  - JSON/YAML 持久化

### 之后 (P3)

- [ ] **插件加载器基础** - 插件发现和生命周期
- [ ] **LSP 服务器集成** - hover, completion 功能

---

## 任务优先级矩阵

| 优先级 | 任务           | 状态      | 预计工作量 |
| ------ | -------------- | --------- | ---------- |
| P1     | 错误消息优化   | 🔜 待开始 | 1-2 天     |
| P2     | 元数据管理器   | 📋 规划中 | 3-5 天     |
| P2.5   | 插件加载器基础 | 📋 规划中 | 3-5 天     |
| P3     | LSP 服务器集成 | 📋 规划中 | 5-7 天     |

---

## 暂缓任务 (详见 BACKLOG.md)

- 稳健性: RPC 超时、Bridge 崩溃日志收集
- 体验: 配置热重载
- 性能: Bridge 预热 (Warmup)
- 运维: CI/CD 流水线

---

## 快速命令参考

```bash
# 开发环境
direnv allow                    # 加载 Nix 环境

# 构建
./xtask.sh build-js             # 构建 JS bridges
cargo build                     # Rust 构建

# 测试
cargo test                      # 所有 Rust 测试
./xtask.sh e2e                  # E2E 测试 (14 场景)

# Lint (应该全部通过)
cargo clippy --all-targets      # Rust: 0 warnings ✅
pnpm exec eslint .              # TS: 0 errors ✅
```

---

## 历史完成记录

完整历史请参见：[COMPLETED_TASKS.md](COMPLETED_TASKS.md)

**最近完成 (2026-02-03)**:

- ✅ Rust Clippy 全部修复 (30+ → 0)
- ✅ TypeScript ESLint 全部修复 (42 → 0)
- ✅ E2E 测试 14/14 通过
- ✅ `/sync-docs` command 创建
