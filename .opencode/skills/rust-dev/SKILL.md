---
name: rust-dev
description: Rust development conventions for NovelSaga core and CLI - use when modifying Rust code in projects/core or projects/cli
---

# NovelSaga Rust Development

## Quick Reference

- **Edition 2024** with nightly features (`#![feature(mpmc_channel)]`)
- **Tests in same file**: `#[cfg(test)] mod tests { ... }`
- **Type exports to TS**: `#[derive(TS)]` + `#[ts(export, export_to = "_config.ts")]`

## State Management

```rust
// ✅ CORRECT: Initialize before get
Initializer::init(Feature::Config);
let state = Initializer::get::<ConfigState>();

// ❌ WRONG: Will panic!
let state = Initializer::get::<ConfigState>();
```

## Build Commands

| Task       | Command                         |
| ---------- | ------------------------------- |
| Build core | `cargo build -p novelsaga-core` |
| Build CLI  | `cargo build -p novelsaga-cli`  |
| Test all   | `cargo test`                    |
| Lint       | `cargo clippy --all-targets`    |

## Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror::Error` for library errors
- Always add context: `.context("loading config file")`

## Anti-Patterns

| Don't                             | Do Instead                  |
| --------------------------------- | --------------------------- |
| Separate test files               | `#[cfg(test)]` in same file |
| `Initializer::get()` without init | Call `init()` first         |
| `println!` in libraries           | Use `tracing` macros        |

## When to Use

Load this skill when working on:

- `projects/core/src/**/*.rs`
- `projects/cli/src/**/*.rs`
- Any Rust test files
