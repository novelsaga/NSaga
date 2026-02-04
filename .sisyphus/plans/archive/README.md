# Archived Plans

本目录包含已完成的工作计划，用于历史参考和知识传承。

## 归档标准

符合以下条件的 plan 应移至此目录：

1. **完全完成**：所有核心目标已达成
2. **不再活跃**：不需要进一步的开发工作
3. **历史参考**：主要用于回顾和知识查询

## 已归档计划

### docs-migration.md

- **完成日期**: 2026-02-04
- **目标**: 将 AI 文档从 `.opencode/docs/` 迁移到 `.sisyphus/` OpenCode 标准结构
- **成果**:
  - ✅ 创建 3 个 plan 文件 (cli-next-steps, completed-stability-quality, future-enhancements)
  - ✅ 初始化 3 个 notepad 目录 (12 个文件)
  - ✅ 更新 `.gitignore`
  - ✅ 删除原始 `.opencode/docs/` 目录

### completed-stability-quality.md

- **时间范围**: 2026-01-26 至 2026-02-04
- **性质**: 历史完成记录归档
- **内容**:
  - 代码质量清理 (30+ Clippy warnings → 0)
  - TypeScript 错误修复 (42 ESLint errors → 0)
  - E2E 测试修复 (14/14 通过)
  - 构建系统优化 (xtask, 类型生成, 增量构建)
  - CLI Runtime & Config 重构
- **相关 notepads**:
  - `.sisyphus/notepads/stability-and-quality/` (8.7KB learnings, 3.5KB issues)
  - `.sisyphus/notepads/cli-runtime-config/` (3KB decisions)
  - `.sisyphus/notepads/build-automation/` (3KB decisions)

## 活跃计划

活跃的工作计划保留在 `.sisyphus/plans/` 根目录：

- `cli-next-steps.md` - CLI 核心功能开发路线图 (P1/P2/P3)
- `future-enhancements.md` - 长期功能 backlog
- `p2-metadata-manager-breakdown.md` - P2 任务细分 (32 tasks)
- `p3-lsp-integration-breakdown.md` - P3 任务细分 (28 tasks)

## 使用归档

### 查询历史决策

```bash
# 搜索特定主题
grep -r "ConfigLoader" .sisyphus/plans/archive/

# 查看完整任务记录
cat .sisyphus/plans/archive/completed-stability-quality.md
```

### 提取最佳实践

归档的 plan 关联的 notepads 包含宝贵的知识：

```bash
# 查看稳定性提升的经验教训
cat .sisyphus/notepads/stability-and-quality/learnings.md

# 查看架构决策
cat .sisyphus/notepads/cli-runtime-config/decisions.md
```

## 注意事项

- **不要删除**：归档文件是项目历史的一部分，应永久保留
- **只读参考**：归档文件不应再被修改（除非发现错误）
- **Git 跟踪**：所有归档 plan 都应提交到 git（与 notepads 不同）
