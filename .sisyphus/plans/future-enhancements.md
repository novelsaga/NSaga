# Future Enhancements & Backlog

## TL;DR

Migrate deferred items (robustness, performance, infrastructure) from `.opencode/docs/BACKLOG.md` to the official OpenCode `.sisyphus/` directory structure as a long-term roadmap.

## Context

### Background

The NovelSaga project is currently focusing on core functionality (metadata, plugin system). Non-critical improvements and future feature ideas are stored in a backlog to avoid scope creep while ensuring these valuable thoughts are not lost.

### Current State

- **Source location**: `.opencode/docs/BACKLOG.md`
- **Content**: 6 deferred items categorized into Robustness, Performance, and Infrastructure.
- **Items**:
  - Robustness: RPC timeout, Crash logging, Config hot reload
  - Performance: Bridge warmup
  - Infrastructure: CI/CD pipeline, Detailed documentation

### Constraints

- **Preserve content**: All original Chinese text, requirement descriptions, proposed solutions, and status/effort estimates must be maintained.
- **Reference-only**: This plan serves as a roadmap; items are currently deferred ("暂缓" or "挂起").
- **Relative paths**: Use relative paths for all file references.
- **Standard format**: Follow OpenCode plan structure (TL;DR, Context, Objectives, TODOs).

## Objectives

### Primary Goal

Successfully migrate all backlog items to `.sisyphus/plans/future-enhancements.md` to maintain project roadmap visibility within the new documentation structure.

### Success Criteria

- [ ] File `.sisyphus/plans/future-enhancements.md` created.
- [ ] All 6 original backlog items included.
- [ ] Original Chinese categories and descriptions preserved.
- [ ] Format follows OpenCode conventions (Context, Objectives, TODOs).
- [ ] No items are marked as complete (use `- [ ]`).

## TODO List

### 稳健性增强 (原 P2 计划)

- [ ] **RPC 调用超时机制**
  - **需求**: 防止 Bridge 进程死锁导致 CLI 永久阻塞。
  - **方案**: `RpcClient::call_with_timeout` (独立线程 + channel)。
  - **状态**: 暂缓。目前本地 IPC 通信较快，死锁概率低。
  - **Est**: 1-2天

- [ ] **Bridge 崩溃日志收集**
  - **需求**: 捕获 Node.js 进程崩溃时的 stderr。
  - **方案**: 后台线程收集 stderr ring buffer。
  - **状态**: 暂缓。开发阶段可以直接看终端输出，或手动复现。
  - **Est**: 1天

- [ ] **配置热重载**
  - **需求**: 修改 `.novelsaga.js` 后无需重启 LSP 即可生效。
  - **方案**: `notify` crate 监听文件变化 + 状态重置。
  - **状态**: 暂缓。目前重启 LSP 速度很快，且配置变更频率不高。
  - **Est**: 2天

### 性能优化 (原 P3 计划)

- [ ] **Bridge 预热 (Warmup)**
  - **需求**: 消除首次请求的 200ms 冷启动延迟。
  - **方案**: CLI 启动时异步唤起 Bridge 进程。
  - **状态**: 暂缓。目前的启动延迟在可接受范围内。
  - **Est**: 1天

### 基础设施

- [ ] **CI/CD 流水线**
  - **需求**: 自动构建、测试、发布。
  - **状态**: **挂起**。用户明确指示："现在发布正式版让人笑掉大牙的"。等功能完备后再搭建。
  - **Est**: 3-5天

- [ ] **详细文档**
  - **需求**: 用户手册和详细的开发指南。
  - **状态**: 暂缓。API 仍在快速迭代中。
  - **Est**: 持续进行
