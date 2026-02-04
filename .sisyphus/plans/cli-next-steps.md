# Plan: NovelSaga CLI Next Steps Migration

## TL;DR

从 `.opencode/docs/NEXT_STEPS.md` 迁移 NovelSaga CLI 的后续开发计划，建立标准化的 OpenCode 任务跟踪。

## Context

- **更新时间**: 2026-02-05
- **当前状态**: ✅ 迁移任务已完成，开发任务已拆分，等待用户规划
- **源文件**: `.opencode/docs/NEXT_STEPS.md`
- **计划性质**: 路线图文档 + 任务拆分完成

## Objectives

- [x] 创建 `.sisyphus/plans/cli-next-steps.md`
- [x] 完整保留 P1/P2/P3 优先级任务
- [x] 保持中文描述及上下文信息
- [x] 集成快速命令参考
- [x] 拆分 P2/P3 为细粒度任务计划

## TODO List

### 立即开始 (P1)

- [x] **错误消息优化** - 改进用户友好的错误提示
  - 文件: `projects/cli/src/bridge/error.rs`, `assets/error.rs`
  - 目标: 从 "IO Error" → "未找到 Node.js，请使用 --node-path 指定路径"
  - 完成: 2026-02-04, commit 8fa7950

### 接下来 (P2)

- [ ] **元数据管理器** - 小说元数据模型和存储
  - 核心数据结构设计
  - 内存存储和查询
  - JSON/YAML 持久化
  - **详细任务拆分**: 见 `p2-metadata-manager-breakdown.md` (32 个细粒度任务)

### 之后 (P3)

- [ ] **插件加载器基础** - 插件发现和生命周期
  - 暂缓：等待用户规划
- [ ] **LSP 服务器集成** - hover, completion 功能
  - **详细任务拆分**: 见 `p3-lsp-integration-breakdown.md` (28 个细粒度任务)
  - 依赖：P2 元数据管理器完成

### 任务优先级矩阵 (参考)

| 优先级 | 任务           | 状态      | 预计工作量 | 细分计划                           |
| ------ | -------------- | --------- | ---------- | ---------------------------------- |
| P1     | 错误消息优化   | ✅ 完成   | 1-2 天     | -                                  |
| P2     | 元数据管理器   | ⏸️ 暂停   | 3-5 天     | `p2-metadata-manager-breakdown.md` |
| P2.5   | 插件加载器基础 | ⏸️ 待规划 | 3-5 天     | 待用户规划                         |
| P3     | LSP 服务器集成 | ⏸️ 暂停   | 5-7 天     | `p3-lsp-integration-breakdown.md`  |

**注**: P2 和 P3 任务已拆分为细粒度子任务（一个文件一个任务），等待用户规划后开始执行。

## Blockers

### P2 和 P3 开发任务

- **状态**: ⏸️ Blocked - 等待用户审查和规划
- **原因**: 用户明确指示暂停，需要审查任务拆分后再决定执行方式
- **已完成准备**:
  - ✅ 任务拆分完成 (P2: 32 tasks, P3: 28 tasks)
  - ✅ 代码模式研究完成
  - ✅ 最佳实践研究完成
- **Unblock 条件**: 用户审查任务拆分并给出执行指令
- **详情**: 见 `.sisyphus/notepads/cli-next-steps/blockers.md`

### 暂缓任务

- **状态**: ⏸️ Deferred - 低优先级增强功能
- **原因**: 依赖核心功能完成，不影响当前开发流程
- **详情**: 见 `future-enhancements.md`

### 暂缓任务 (详见 BACKLOG.md)

- [ ] 稳健性: RPC 超时、Bridge 崩溃日志收集
- [ ] 体验: 配置热重载
- [ ] 性能: Bridge 预热 (Warmup)
- [ ] 运维: CI/CD 流水线

## Quick Command Reference

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
