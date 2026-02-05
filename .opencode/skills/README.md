# OpenCode Skills for NovelSaga

Welcome to the NovelSaga OpenCode skills directory! These skills provide AI agents with specialized knowledge and best practices for working on different aspects of the NovelSaga project.

## Quick Selection Guide

Use this table to find the right skill for your task:

| Skill                    | Description                                                                                                                    | When to Use                                                                                       |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| **core-dev**             | Rust core library development - state management, config, type exports                                                         | Modifying core library state, configuration systems, or type exports in `projects/core/`          |
| **cli-dev**              | CLI and bridge management development - use when modifying CLI commands, bridge system, or runtime management                  | Working on CLI commands, bridge system management, or runtime discovery in `projects/cli/`        |
| **lsp-dev**              | LSP protocol implementation for NovelSaga - use when modifying LSP backend, document handlers, or language server capabilities | Implementing LSP features, document sync, or language server capabilities                         |
| **ts-bridge**            | TypeScript bridge development for JSON-RPC services                                                                            | Modifying TypeScript bridges, JSON-RPC services, or CLI integration in `projects/cli-js-bridges/` |
| **ffi-diplomat**         | FFI bindings using Diplomat - use when modifying diplomat_ffi.rs or exposing Rust types to C/WASM                              | Working on FFI bindings, Diplomat patterns, or cross-platform C/WASM exposure                     |
| **nix-env**              | Nix development environment setup and management for NovelSaga                                                                 | Setting up development environment, direnv configuration, or environment troubleshooting          |
| **nix-build**            | Nix build system for reproducible builds and multi-platform compilation                                                        | Building via Nix, updating hashes, or multi-platform compilation                                  |
| **nix-workflow**         | Complete Nix development environment and build system                                                                          | Complete Nix workflows combining environment and build tasks                                      |
| **testing-guide**        | NovelSaga-specific testing patterns - same-file tests, E2E structure, test matrix                                              | Writing tests, understanding test structure, or running test suites                               |
| **project-architecture** | NovelSaga module relationships, data flow, and system initialization sequence                                                  | Understanding system design, module interactions, or data flow                                    |
| **end-to-end-workflow**  | Decision tree for feature development - determine which files to modify for different feature types                            | Planning a new feature or understanding which files to change                                     |
| **docs-maintenance**     | Documentation maintenance and plan archiving - guides AI on when to update docs and managing completed plans                   | Updating documentation, maintaining AGENTS.md, or archiving completed plans                       |
| **rust-dev**             | Rust development conventions for NovelSaga core and CLI                                                                        | General Rust development in the project                                                           |

## Usage Examples

### Example 1: Adding a new LSP feature

```
Load skills: [lsp-dev, cli-dev, core-dev]
Reason: Need to understand LSP protocol, CLI bridge integration, and core state management
```

### Example 2: Setting up development environment

```
Load skills: [nix-env, nix-build]
Reason: Need environment setup and build verification
```

### Example 3: Implementing a new feature

```
Load skills: [end-to-end-workflow, project-architecture, testing-guide]
Reason: Plan feature scope, understand system design, verify with tests
```

### Example 4: Adding cross-platform support

```
Load skills: [ffi-diplomat, core-dev, nix-build]
Reason: Diplomat FFI patterns, core library changes, and Nix multi-platform builds
```

## Skill Organization

All skills follow a consistent format:

- **Quick Reference** - Essential commands, patterns, and key concepts
- **Anti-Patterns** - Common mistakes and what to avoid
- **When to Use** - Clear guidance on when this skill is applicable
- **Code References** - Links to relevant source files and examples

## Finding the Right Skill

### By Codebase Location

- `projects/core/` → **core-dev**, **ffi-diplomat**
- `projects/cli/` → **cli-dev**, **lsp-dev**
- `projects/cli-js-bridges/` → **ts-bridge**
- Build & Environment → **nix-env**, **nix-build**
- Tests → **testing-guide**
- Documentation → **docs-maintenance**

### By Task Type

- **Understand the system** → **project-architecture**, **end-to-end-workflow**
- **Implement a feature** → **end-to-end-workflow** + specific component skills
- **Debug an issue** → Component skill + **project-architecture**
- **Set up environment** → **nix-env**, **nix-build**
- **Write tests** → **testing-guide**
- **Maintain docs** → **docs-maintenance**

## Skill Development

When adding new skills:

1. Place skill file at `.opencode/skills/{name}/SKILL.md`
2. Follow the format from existing skills
3. Include Quick Reference, Anti-Patterns, and When to Use sections
4. Add concrete code examples and references
5. Update this README with the new skill entry

See **docs-maintenance** skill for more information about maintaining OpenCode skills.
