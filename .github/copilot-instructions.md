# NovelSaga Development Guide

## Project Overview

**Multi-Editor Novel Editing System Backend** - A backend for novel/fiction editing supporting multiple platforms:
- **LSP Server** (preferred): For editors with LSP support (VSCode, Neovim, etc.)
- **C Library** (via Diplomat FFI): For iOS, Android, Web (via .so and WASM)
- Communication priority: LSP first, then C-lib bindings

### Client Architecture

```
Core (novelsaga_core) - Defines config, plugin system, article processing
  ↓
LSP Server (projects/cli) - Official client, implements config loaders
  ↓
Editors (VSCode, Neovim) - LSP clients, provide UI
```

### Core Features (see [需求总结](https://github.com/novelsaga/plans/blob/main/%E9%9C%80%E6%B1%82%E6%80%BB%E7%BB%93.md))

1. **Article Processing**: AST parsing, segmentation, tokenization, vectorization
2. **Plugin System**: JS plugins, WASI plugins, dynamic libraries
3. **Caching**: Segment-level and full-document caching
4. **Outline Management**: Fine and coarse outlines
5. **Export**: Typst for PDF export
6. **Charts**: Relationship graphs, timelines, writing statistics
7. **Git Integration**: Version control support
8. **Platform Plugins**: Support for platforms like Qidian (起点)
9. **Metadata**: UUID-based (v7), stored in `.novelsaga/` hidden folder (TODO: detailed design)
10. **Hierarchical Config**: Layered configuration with hot-reload, supports TS/JS hooks
11. **FS Abstraction**: Local, Android, Web, WebDAV file systems
12. **Hybrid Programming**: Functional + OOP paradigm

### LSP Features (for Chinese Web Novel Writing)

The LSP server provides rich language services for Markdown-based novel writing:

- **Completion**: Character names, location names, custom vocabularies
- **Go to Definition**: Jump to character/location definitions, settings
- **Character Biography**: Auto-generated character life summaries
- **Timeline Visualization**: Visual world timeline and event tracking
- **AI Integration**: Built-in AI assistance for writing suggestions
- **MCP Support**: Model Context Protocol integration
- **Plugin System**: Extensible plugin architecture for custom features
- **Sensitive Word Detection**: Flag potentially problematic content
- **Character Consistency**: Warn about character trait inconsistencies
  - Detect unreasonable changes (errors)
  - Allow reasonable character growth (valid changes)

These features are tailored for Chinese online novel writing workflows and requirements.

## Development Environment

### ⚠️ CRITICAL: Use Nix

**You MUST use Nix + direnv for development. Period.**

Not using Nix? Prepare for dependency hell, version mismatches, and "works on my machine" syndrome. Don't come crying when your build breaks. You've been warned. 后果自负。

**Setup:**
```bash
# Install Nix (if you haven't already)
sh <(curl -L https://nixos.org/nix/install) --daemon

# Enable flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Install direnv
nix-env -iA nixpkgs.direnv

# Hook direnv into your shell (add to ~/.zshrc or ~/.bashrc)
eval "$(direnv hook zsh)"  # or bash

# Enter project directory (direnv will auto-load)
cd ~/code/src/nsaga
direnv allow  # First time only
```

The `.envrc` file automatically loads the Nix development environment with all correct dependencies, versions, and tools. Don't fight it, just use it.

## Architecture Overview

**Hybrid Rust + TypeScript/JavaScript System** with cross-language communication:

- **Rust Core** (`projects/core/`): State management, config, article/library processing
  - **Article**: Article objects with content and metadata (cache location, vector structure)
  - **Library**: Utility functions and structs for functionality implementation
  - Uses `OnceLock` for global singleton state (`state/init.rs`)
  - Config types exported to TypeScript via `ts-rs` (generates `_config.ts`)
  - Diplomat FFI for C bindings (iOS/Android/Web via .so and WASM)

- **Rust CLI** (`projects/cli/`): LSP server, plugin system, runtime detection
  - Node.js version detection for TypeScript support (≥23.6 for native TS)
  - JS Bridge system for dynamic config loading via JSON-RPC over stdin/stdout

- **TypeScript Bridges** (`projects/cli/assets/js/src/`): Node.js services called by Rust
  - `shared/`: JSON-RPC 2.0 protocol, stdio transport, bridge server framework
  - `config-bridge/`: Loads JS/TS config files and returns serialized config to Rust
  - Standard structure: `src/`, `build.mts`, `package.json`, `tsconfig.json`

### Cross-Language Communication Flow

```
Rust CLI → spawn Node.js → stdin/stdout → JSON-RPC 2.0 → Bridge Server
         ← JSON response ← parse/serialize ← Service Handler ← import(config.js)
```

## Configuration System

### Config File Discovery

The system searches for config files in this order (from current directory upward to workspace root):

**File names** (without extension):
- `novelsaga.config`
- `.novelsaga`

**Supported formats** (via `config` crate):
- `.corn`, `.ron`, `.toml`, `.yaml`, `.yml`, `.json5`, `.json`, `.ini`
- `.js`, `.mjs`, `.cjs` (if JS support enabled)
- `.ts`, `.mts`, `.cts` (if TS support enabled)

**Ignore files**:
- `.novelsagaignore`, `.nsignore`, `.novelsaga.ignore`, `.ns.ignore`

### Hierarchical Config Loading

1. **Root Config** (`RootConfig`): Loaded once from workspace root, immutable
   - Contains `workspace` settings (cache_dir, novelsaga_dir)
   - Located via upward search from `current_dir()` until found

2. **Overridable Config** (`OverridableConfig`): Loaded per-directory, cached
   - Contains `fmt` (formatter settings)
   - Merged with parent directory configs (layered override)
   - Cached in `ConfigManager` for performance

3. **Final Config** (`NovelSagaConfig`): `RootConfig` + `OverridableConfig` (flattened)

### Config File Structure

```javascript
// novelsaga.config.js - supports both static and function forms
export default {
  workspace: {
    cache_dir: ".novelsaga/cache",
    novelsaga_dir: ".novelsaga"
  },
  fmt: { /* formatter options */ }
};

// or functional form with dynamic settings
export default (settings) => ({
  workspace: {
    cache_dir: `${settings.PROJECT_ROOT}/.novelsaga/cache`
  },
  fmt: {}
});
```

## Key Conventions

### Rust

- **Edition 2024**, nightly features (`#![feature(mpmc_channel)]`)
- Workspace structure: `projects/*` + `xtask/` for build tasks
- Global state via `Initializer::init(feature)` - must be called before accessing state
- Type exports to TS: Add `#[derive(TS)]` and `#[ts(export, export_to = "_config.ts")]`
- Serde with `#[serde(flatten)]` for config merging (`RootConfig` + `OverridableConfig`)
- **Tests**: Write tests in the same file as source code using `#[cfg(test)]` modules

### TypeScript/JavaScript

- **Node.js ≥20**, ESM modules (`"type": "module"`)
- Monorepo managed by `pnpm workspace` (root: `~/code/src/nsaga/pnpm-workspace.yaml`)
- **Never install deps at subproject level** - use root `pnpm install`
- Build with `node --experimental-strip-types build.mts` (Node 23.6+) or esbuild
- Bridge outputs: Single bundled JS to `projects/cli/assets/js/dist/`

### JS Bridge Development Pattern

Each bridge follows this structure:
```typescript
// src/index.ts
import { BridgeServer } from "@nsaga/bridge-nodejs/bridge-server";
import { MyService } from "./services/my-service.js";

const server = new BridgeServer({ name: "my-bridge", version: "1.0.0" });
server.registerService("myservice", new MyService());
server.start();
```

Services implement JSON-RPC methods via `"service.method"` routing:
```typescript
import type { Service } from "@nsaga/bridge-core/interfaces/service";

class MyService implements Service {
  [method: string]: any; // Index signature for Service interface
  async myMethod(params: unknown): Promise<Result> { /* ... */ }
}
```

Rust calls via: `{"jsonrpc":"2.0","id":1,"method":"myservice.myMethod","params":{}}`

### Bridge Architecture (Runtime Adapters)

The bridge system uses a layered architecture to support multiple JS runtimes:

- **bridge-core** (`@nsaga/bridge-core`): Runtime-agnostic core
  - `types/rpc.ts`: JSON-RPC 2.0 protocol types
  - `interfaces/transport.ts`: Abstract Transport interface
  - `interfaces/service.ts`: Abstract Service interface

- **bridge-nodejs** (`@nsaga/bridge-nodejs`): Node.js runtime adapter
  - `transport.ts`: StdioTransport implementation (stdin/stdout)
  - `rpc-handler.ts`: RPCHandler with service routing
  - `bridge-server.ts`: BridgeServer main class
  - Depends on: `@nsaga/bridge-core`

- **Future adapters**: `bridge-bun`, `bridge-deno` (same pattern)

- **Bridges** (e.g., `config-bridge`): Specific functionality bridges
  - Depend on runtime adapter (e.g., `@nsaga/bridge-nodejs`)
  - Build to single file: `dist/{bridge-name}.js`

## Critical Workflows

### Building

```bash
# Root install (required before any build)
cd ~/code/src/nsaga && pnpm install

# Build bridge-core first (dependency of runtime adapters)
cd projects/cli/assets/js/src/bridge-core && pnpm run build

# Build runtime adapter (e.g., bridge-nodejs)
cd ../bridge-nodejs && pnpm run build

# Build specific bridge
cd ../config-bridge && pnpm run build

# Rust build
cd ~/code/src/nsaga && cargo build
```

### Using xtask

```bash
# View available tasks
./xtask.sh --help
# or
$DEVENV_ROOT/xtask.sh --help
```

### Testing JS Bridges

```bash
# Manual test via stdin/stdout
export NSAGA_CLI_CONFIG_PATH="/path/to/config.js"
echo '{"jsonrpc":"2.0","id":1,"method":"config.get","params":{}}' | \
  node projects/cli/assets/js/dist/config-bridge.js
```

## Integration Points

### Rust ↔ TypeScript Type Sync

When modifying config structs in `projects/core/src/config/mod.rs`:
1. Add `#[derive(TS)]` and `#[ts(export, export_to = "_config.ts")]`
2. Run tests to generate: `cargo test export_bindings`
3. TS types appear in `projects/cli/assets/js/src/config-bridge/src/types/_config.ts`
4. Extend in `config.ts`: `type NovelSagaConfig = OverridableConfig & RootConfig`

### Adding New JS Bridges

1. Create `projects/cli/assets/js/src/my-bridge/` with standard 4-file structure
2. Add to `pnpm-workspace.yaml` packages pattern (already covered by `src/*`)
3. Add `"@nsaga/bridge-nodejs": "workspace:*"` dependency in `package.json`
4. Build outputs to `dist/my-bridge.js` (sibling to `config-bridge.js`)
5. Rust spawns via `RuntimeProcess::spawn(info, "path/to/dist/my-bridge.js", env)`

## Cross-Platform Considerations

- **Path handling**: Use `PathBuf` and `resolve()` - Windows uses `\`, Unix `/`
- **Process management**: `Command::new()` needs `.exe` on Windows for Node/Bun/Deno
- **JS runtime detection**: Check `node`, `bun`, `deno` executables in PATH + version managers (nvm, nvm-windows, scoop, homebrew)
- **Stdio transport**: Works identically on Unix and Windows (UTF-8 line-based)

## Documentation Organization

- **AI-generated docs**: Place in `docs/ai-generated/` (not scattered across project)
- **Manual docs**: Place in `docs/` root or relevant project subdirectories

## Working with AI Agents

### Communication Preferences

When working on this project, AI agents should:

1. **Seek Confirmation**: Always ask for clarification before making significant changes
   - Uncertain about requirements? Ask first, don't guess
   - Multiple valid approaches? Present options and let user choose
   - Before installing new dependencies? List them and get approval

2. **Interactive Development**: Engage in dialogue rather than assuming
   - Stop and ask when encountering ambiguity
   - Confirm architectural decisions before implementation
   - Validate assumptions about existing patterns

3. **Documentation**: Keep AI-generated documentation organized
   - All AI-generated docs go to `docs/ai-generated/`
   - Never scatter documentation across the project
   - Update existing docs rather than creating duplicates

4. **Incremental Progress**: Work step-by-step with checkpoints
   - Break complex tasks into smaller steps
   - Confirm completion of each step before proceeding
   - Provide progress updates for multi-step work

5. **Specific Context**: When asking questions, include:
   - Specific file paths being discussed
   - Concrete examples from the codebase
   - Clear reasoning for suggested approaches

## Common Pitfalls

- **Don't modify `_config.ts`** - regenerated by Rust, extend in separate files
- **Rust state initialization**: Must call `Initializer::init()` before `Initializer::get()`
- **Bridge imports**: Use `@nsaga/bridge-nodejs/...` or `@nsaga/bridge-core/...` not relative paths outside rootDir
- **TypeScript workspace types**: If importing types from transitive dependencies (e.g., `bridge-core` types in `config-bridge`), add as devDependency for TypeScript resolution
- **Node version**: Native TS requires ≥23.6, check with `cli.is_ts_supported()`
- **JSON-RPC errors**: Stderr for logs, stdout only for JSON responses
- **Test location**: Write tests in same file as source code, don't separate far away
- **Config file location**: Search upward from current directory, stop at workspace root
- **Dependencies**: Always install at root level (`~/code/src/nsaga`), never in subprojects
- **Shell aliases**: When debugging, check `command -v <cmd>` (e.g., `ls→eza`, `cat→bat`, `find→fd`) - use `list_dir`, `read_file` tools instead
