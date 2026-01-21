# JS Config Loader 实现路线图

## 项目目标

实现一个通过 JSON-RPC 桥接 Rust 和 JavaScript 的配置加载系统，支持动态加载 JS/TS 配置文件，并集成到 NovelSaga 的初始化流程中。

---

## 阶段 0: 架构设计 ✅ (已完成)

### 0.1 系统架构概览

**整体架构**：分层的跨语言桥接系统

```
┌─────────────────────────────────────────────────────────────┐
│                      Rust 侧 (CLI)                           │
├─────────────────────────────────────────────────────────────┤
│  ConfigLoader                                                │
│    ↓                                                         │
│  BridgeManager ─→ Bridge Instance (NodeBridge/BunBridge)   │
│    ↓                                                         │
│  RPCClient ─→ StdioTransport (stdin/stdout)                │
│    ↓                                                         │
│  RuntimeProcess (spawn node/bun/deno)                       │
└──────────────────┬──────────────────────────────────────────┘
                   │ JSON-RPC 2.0 over stdin/stdout
┌──────────────────┴──────────────────────────────────────────┐
│                   JavaScript 侧 (Bridge)                     │
├─────────────────────────────────────────────────────────────┤
│  config-bridge.js (Entry Point)                             │
│    ↓                                                         │
│  BridgeServer ──────────────────────────────────────────┐  │
│    │                                                      │  │
│    ├─→ StdioTransport (stdin/stdout)                    │  │
│    │                                                      │  │
│    └─→ RPCHandler                                        │  │
│         ↓                                                 │  │
│       ConfigService ─→ import() ─→ User Config File     │  │
│                                      (.js/.mjs/.cjs)      │  │
└─────────────────────────────────────────────────────────────┘
```

### 0.2 分层架构设计

#### **Layer 1: 核心抽象层 (bridge-core)**
**职责**: 定义运行时无关的类型和接口

- **JSON-RPC 2.0 协议类型**
  - `RPCRequest`, `RPCResponse`, `RPCNotification`
  - `RPCErrorCode` 标准错误码
  - 类型守卫函数 (TypeScript)

- **Transport 抽象接口**
  ```typescript
  interface Transport {
    send(message: RPCResponse): void;
    onMessage(handler: MessageHandler): void;
    close(): void;
  }
  ```

- **Service 抽象接口**
  ```typescript
  interface Service {
    [method: string]: any; // 支持动态方法调用
  }
  ```

**设计原则**:
- 完全运行时无关
- 仅包含类型定义和接口
- 为所有运行时适配器提供统一基础

#### **Layer 2: 运行时适配器层 (bridge-nodejs, bridge-bun, bridge-deno)**
**职责**: 实现特定运行时的 Transport 和 Server

**以 bridge-nodejs 为例**:

- **StdioTransport** (实现 Transport 接口)
  - 使用 `node:readline` 读取 stdin
  - 使用 `console.log` 输出到 stdout
  - 逐行解析 JSON-RPC 消息
  - 错误输出到 stderr (不干扰 JSON-RPC)

- **RPCHandler**
  - 服务注册: `registerService(name, service)`
  - 方法路由: `"service.method"` 格式
  - 请求分发到服务方法
  - 错误响应生成

- **BridgeServer**
  - 整合 Transport + RPCHandler
  - 生命周期管理 (start/stop)
  - 异常捕获和进程退出处理

**适配器特点**:
- Node.js: 基于 `node:*` 内置模块，无外部依赖
- Bun: 未来可利用 Bun 特性 (如原生 TypeScript)
- Deno: 未来需适配 Deno 的权限模型

#### **Layer 3: 功能桥层 (config-bridge, 未来: plugin-bridge)**
**职责**: 实现具体业务逻辑

**config-bridge 架构**:

```typescript
// 主入口
import { BridgeServer } from "@nsaga/bridge-nodejs/bridge-server";
import { ConfigService } from "./services/config.js";

const server = new BridgeServer({ name: "config-bridge", version: "1.0.0" });
server.registerService("config", new ConfigService());
server.start();
```

**ConfigService 职责**:
1. 环境变量读取 (`NSAGA_CLI_*`)
2. 动态导入配置文件 (`import()`)
3. 处理 ESM/CJS 差异
4. 执行函数式配置
5. 配置净化 (移除不可序列化内容)

**配置文件支持矩阵**:

| 格式 | 扩展名 | 静态对象 | 函数 | Node.js | Bun | Deno |
|------|--------|---------|------|---------|-----|------|
| ESM  | .js/.mjs | ✅ | ✅ | ✅ | ✅ | ✅ |
| CJS  | .cjs     | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| TypeScript | .ts/.mts/.cts | ✅ | ✅ | ≥23.6 | ✅ | ✅ |

### 0.3 通信协议设计

#### **JSON-RPC 2.0 选型理由**
- 标准化协议，跨语言支持广泛
- 简单的请求/响应模型
- 支持批量请求 (未来扩展)
- 明确的错误码规范

#### **传输层: stdin/stdout**

**选择理由**:
- 跨平台支持 (Windows/Unix)
- 进程间通信简单可靠
- 无需网络端口，避免安全隐患
- 父进程完全控制子进程生命周期

**协议格式**:
```
stdin:  {"jsonrpc":"2.0","id":1,"method":"config.get","params":{}}\n
stdout: {"jsonrpc":"2.0","result":{...},"id":1}\n
stderr: [日志输出，不影响 JSON-RPC]
```

**关键约定**:
- 每条消息一行 (以 `\n` 分隔)
- stdout 仅用于 JSON-RPC 响应
- stderr 用于日志、调试信息、启动通知
- 消息必须是有效的 JSON-RPC 2.0 格式

### 0.4 Rust 侧架构设计

#### **模块划分**

```
projects/cli/src/bridge/
├── mod.rs                    # 模块导出
├── rpc/                      # JSON-RPC 协议层
│   ├── mod.rs
│   ├── types.rs             # RpcRequest, RpcResponse 等
│   └── client.rs            # RpcClient 实现
├── transport/               # 传输层抽象
│   ├── mod.rs
│   └── stdio.rs            # StdioTransport 实现
├── runtime/                # 运行时管理
│   ├── mod.rs
│   ├── discovery.rs        # 运行时检测
│   └── process.rs          # RuntimeProcess 实现
└── manager/                # Bridge 管理
    ├── mod.rs
    ├── bridge.rs           # Bridge trait
    └── manager.rs          # BridgeManager 实现

projects/cli/src/config/
└── loader.rs               # ConfigLoader (使用 BridgeManager)
```

#### **关键接口设计**

**Transport Trait**:
```rust
pub trait Transport: Send {
    fn send(&mut self, message: &RpcResponse) -> Result<()>;
    fn receive(&mut self) -> Result<Option<RpcRequest>>;
    fn close(&mut self) -> Result<()>;
}
```

**Bridge Trait**:
```rust
pub trait Bridge: Send {
    fn name(&self) -> &str;
    fn call(&mut self, method: &str, params: Value) -> Result<Value>;
    fn is_healthy(&self) -> bool;
}
```

**BridgeManager API**:
```rust
impl BridgeManager {
    pub fn new() -> Self;
    pub fn register(&mut self, name: &str, bridge: Box<dyn Bridge>);
    pub fn call(&mut self, bridge: &str, method: &str, params: Value) -> Result<Value>;
    pub fn shutdown(&mut self) -> Result<()>;
}
```

**ConfigLoader API**:
```rust
impl ConfigLoader {
    pub fn new(bridge_manager: Arc<Mutex<BridgeManager>>) -> Self;
    pub fn discover(&self, start_dir: &Path) -> Result<Option<PathBuf>>;
    pub fn load(&mut self, path: &Path) -> Result<NovelSagaConfig>;
}
```

### 0.5 配置文件设计

#### **配置文件发现规则**

1. **文件名匹配** (不区分大小写):
   - `novelsaga.config.*`
   - `.novelsaga.*`

2. **扩展名优先级**:
   - TypeScript: `.ts`, `.mts`, `.cts` (需运行时支持)
   - JavaScript: `.js`, `.mjs`, `.cjs`
   - 其他: `.toml`, `.json`, `.json5`, `.yaml`, `.ron`, `.corn`

3. **搜索策略**:
   - 从当前目录开始向上搜索
   - 停止条件: 找到配置文件或到达 workspace root
   - Workspace root 标志: `.git/`, `Cargo.toml`, `package.json`

#### **配置文件格式**

**静态对象形式**:
```javascript
// novelsaga.config.js
export default {
  workspace: {
    cache_dir: ".novelsaga/cache",
    novelsaga_dir: ".novelsaga"
  },
  fmt: {
    indent: 2,
    line_width: 100
  }
};
```

**函数形式** (支持动态逻辑):
```javascript
// novelsaga.config.js
export default (settings) => {
  return {
    workspace: {
      cache_dir: `${settings.PROJECT_ROOT}/.novelsaga/cache`,
      novelsaga_dir: ".novelsaga"
    },
    fmt: {
      indent: settings.IS_DEV ? 2 : 0,
      line_width: 100
    }
  };
};
```

**环境变量传递**:
```rust
// Rust 设置环境变量
env.insert("NSAGA_CLI_CONFIG_PATH", config_path.to_string_lossy());
env.insert("NSAGA_CLI_CONFIG_IS_COMMONJS", "false");
env.insert("NSAGA_CLI_PROJECT_ROOT", project_root.to_string_lossy());
env.insert("NSAGA_CLI_IS_DEV", "true");

// JavaScript 读取
const settings = {
  CONFIG_PATH: process.env.NSAGA_CLI_CONFIG_PATH,
  CONFIG_IS_COMMONJS: process.env.NSAGA_CLI_CONFIG_IS_COMMONJS === "true",
  PROJECT_ROOT: process.env.NSAGA_CLI_PROJECT_ROOT,
  IS_DEV: process.env.NSAGA_CLI_IS_DEV === "true",
};
```

### 0.6 跨平台兼容性设计

#### **运行时检测策略**

1. **标准安装路径检测**:
   - Unix: `/usr/bin/node`, `/usr/local/bin/node`
   - Windows: `C:\Program Files\nodejs\node.exe`

2. **PATH 环境变量检测**:
   - 使用 `which` (Unix) 或 `where` (Windows)
   - 检查可执行文件名: `node`, `node.exe`

3. **版本管理器检测**:
   - nvm (Unix): `~/.nvm/versions/node/*/bin/node`
   - nvm-windows: `%APPDATA%\nvm\*\node.exe`
   - fnm: `~/.fnm/node-versions/*/installation/bin/node`
   - volta: `~/.volta/bin/node`

4. **版本验证**:
   ```bash
   node --version  # v20.x.x
   bun --version   # 1.x.x
   deno --version  # deno 2.x.x
   ```

#### **进程管理策略**

- **启动**: 使用 `std::process::Command`
- **监控**: 定期健康检查 (ping-pong 机制)
- **重启**: 失败后自动重试 (最多 3 次)
- **关闭**: 发送 SIGTERM，超时后 SIGKILL

### 0.7 错误处理设计

#### **错误分类**

1. **协议错误** (JSON-RPC 标准错误码):
   - `-32700`: Parse error (JSON 格式错误)
   - `-32600`: Invalid Request (非法请求)
   - `-32601`: Method not found (方法不存在)
   - `-32602`: Invalid params (参数错误)
   - `-32603`: Internal error (内部错误)

2. **业务错误** (自定义错误码 -32000 到 -32099):
   - `-32001`: Service not found (服务不存在)
   - `-32002`: Service error (服务执行错误)
   - `-32003`: Timeout error (超时)

3. **系统错误** (Rust 侧):
   - RuntimeNotFound (未找到运行时)
   - ProcessSpawnError (进程启动失败)
   - CommunicationError (通信失败)
   - ConfigParseError (配置解析失败)

#### **错误恢复策略**

- **进程崩溃**: 自动重启 bridge (最多 3 次)
- **通信超时**: 重试请求 (指数退避)
- **配置错误**: 回退到默认配置或报错退出
- **运行时缺失**: 提示用户安装或使用其他运行时

### 0.8 设计决策记录

#### **为什么选择 JSON-RPC 而不是 gRPC？**
- ✅ 无需 protobuf 定义，开发更灵活
- ✅ 纯文本协议，易于调试
- ✅ 跨语言支持广泛，生态成熟
- ❌ 性能不如 gRPC (但配置加载不是性能热点)

#### **为什么使用 stdin/stdout 而不是 TCP 或 Unix Socket？**
- ✅ 跨平台支持最简单
- ✅ 无需处理端口冲突
- ✅ 父进程完全控制子进程生命周期
- ✅ 无需额外的安全机制
- ❌ 不支持多客户端 (但不需要)

#### **为什么分离 bridge-core 和 bridge-nodejs？**
- ✅ 未来可轻松添加 Bun、Deno 适配器
- ✅ 类型定义可复用
- ✅ 强制接口一致性
- ✅ 符合开闭原则

#### **为什么支持函数式配置？**
- ✅ 允许根据环境变量动态生成配置
- ✅ 支持复杂的条件逻辑
- ✅ 与 Vite、Next.js 等工具一致
- ✅ 不增加实现复杂度

### 完成内容
- ✅ 分层架构设计 (3 层)
- ✅ 通信协议设计 (JSON-RPC 2.0 + stdin/stdout)
- ✅ Rust 侧模块划分和接口设计
- ✅ 配置文件发现和加载策略
- ✅ 跨平台兼容性方案
- ✅ 错误处理策略
- ✅ 设计决策记录

### 输出产物
- ✅ 完整架构设计文档
- ✅ 接口定义和模块划分
- ✅ 配置文件格式规范

---

## 阶段 1: JS Bridge 基础设施 ✅ (已完成)

### 1.1 bridge-core (运行时无关核心)
**位置**: `projects/cli/assets/js/src/bridge-core/`

**完成内容**:
- ✅ JSON-RPC 2.0 类型定义 (`types/rpc.ts`)
  - RPCRequest, RPCResponse, RPCNotification
  - 错误码定义 (RPCErrorCode)
  - 类型守卫函数
- ✅ Transport 抽象接口 (`interfaces/transport.ts`)
  - send(), onMessage(), close()
- ✅ Service 抽象接口 (`interfaces/service.ts`)
  - 索引签名支持动态方法调用
- ✅ 构建系统
  - esbuild 编译 JS
  - tsc 生成 .d.ts 类型声明
- ✅ package.json exports 配置
  - types 字段指向 .d.ts

### 1.2 bridge-nodejs (Node.js 运行时适配器)
**位置**: `projects/cli/assets/js/src/bridge-nodejs/`

**完成内容**:
- ✅ StdioTransport 实现 (`transport.ts`)
  - 基于 readline 的 stdin 读取
  - 基于 console.log 的 stdout 输出
  - 消息处理器注册
- ✅ RPCHandler 实现 (`rpc-handler.ts`)
  - 服务注册和路由 (service.method 格式)
  - 请求/通知处理
  - 错误响应生成
- ✅ BridgeServer 实现 (`bridge-server.ts`)
  - 整合 Transport + RPCHandler
  - 生命周期管理 (start/stop)
  - 异常处理
- ✅ 依赖 bridge-core
- ✅ 构建系统配置
- ✅ 类型声明生成

### 1.3 config-bridge (配置加载桥)
**位置**: `projects/cli/assets/js/src/config-bridge/`

**完成内容**:
- ✅ ConfigService 实现 (`services/config.ts`)
  - 环境变量读取 (NSAGA_CLI_*)
  - 动态 import() 支持 ESM/CJS
  - 函数/静态对象配置支持
  - 配置净化 (移除不可序列化内容)
- ✅ 主入口 (`index.ts`)
  - 创建 BridgeServer
  - 注册 ConfigService
- ✅ 构建配置
  - 单文件打包到 `dist/config-bridge.js`
  - 生成 `dist/package.json` (标记为 ESM)
- ✅ 手动测试
  - ESM 静态配置 ✅
  - ESM 函数配置 ✅
  - CJS 静态配置 ✅
  - CJS 函数配置 ✅

### 1.4 测试文件
**位置**: `projects/cli/assets/test/`

**完成内容**:
- ✅ 测试配置文件
  - `test-config.mjs` (ESM 静态)
  - `test-config-fn.mjs` (ESM 函数)
  - `test-config.cjs` (CJS 静态)
  - `test-config-fn.cjs` (CJS 函数)
- ✅ 导入测试 (`test-import.mjs`)
  - 验证 Node.js 对 CJS/ESM 的处理

### 输出产物
- ✅ `@nsaga/bridge-core` (类型和接口)
- ✅ `@nsaga/bridge-nodejs` (Node.js 适配器)
- ✅ `dist/config-bridge.js` (可执行桥)
- ✅ 测试验证通过

---

## 阶段 2: Rust JSON-RPC 通信层 🚧 (待实现)

### 2.1 JSON-RPC 类型定义
**位置**: `projects/cli/src/bridge/rpc/`

**待完成**:
- [ ] Rust 结构体定义
  - `RpcRequest`, `RpcResponse`, `RpcNotification`
  - `RpcError`, `RpcErrorCode`
- [ ] Serde 序列化/反序列化
  - `#[derive(Serialize, Deserialize)]`
- [ ] 类型转换辅助函数

**依赖**: 无

**预估时间**: 2-3 小时

### 2.2 Transport Trait 定义
**位置**: `projects/cli/src/bridge/transport/mod.rs`

**待完成**:
- [ ] Transport trait 定义
  ```rust
  pub trait Transport {
      fn send(&mut self, message: RpcResponse) -> Result<()>;
      fn receive(&mut self) -> Result<Option<RpcRequest>>;
      fn close(&mut self) -> Result<()>;
  }
  ```
- [ ] StdioTransport 实现
  - stdin 阻塞读取 (BufReader)
  - stdout 行写入
  - 线程安全考虑

**依赖**: JSON-RPC 类型定义

**预估时间**: 3-4 小时

### 2.3 RPC Client 实现
**位置**: `projects/cli/src/bridge/rpc/client.rs`

**待完成**:
- [ ] RpcClient 结构体
  - 持有 Transport
  - 请求 ID 生成
- [ ] 同步调用 API
  ```rust
  pub fn call(&mut self, method: &str, params: Value) -> Result<Value>
  ```
- [ ] 超时处理
- [ ] 错误映射

**依赖**: Transport Trait

**预估时间**: 4-5 小时

### 2.4 集成测试
**位置**: `projects/cli/src/bridge/rpc/tests.rs`

**待完成**:
- [ ] 手动测试与 config-bridge 通信
  - 启动 config-bridge 子进程
  - 发送 JSON-RPC 请求
  - 验证响应
- [ ] 单元测试 (mock transport)

**依赖**: RPC Client

**预估时间**: 2-3 小时

### 输出产物
- [ ] `bridge::rpc` 模块
- [ ] `bridge::transport` 模块
- [ ] 可与 JS bridge 通信的 Rust RPC 客户端

---

## 阶段 3: Rust 运行时管理层 🚧 (待实现)

### 3.1 运行时检测
**位置**: `projects/cli/src/bridge/runtime/discovery.rs`

**待完成**:
- [ ] RuntimeType 枚举
  ```rust
  pub enum RuntimeType {
      NodeJs { version: String },
      Bun { version: String },
      Deno { version: String },
  }
  ```
- [ ] RuntimeDiscovery 实现
  - 检测 PATH 中的运行时
  - 版本检测 (`node --version`, `bun --version`, `deno --version`)
  - 版本管理器检测 (nvm, nvm-windows, fnm, volta)
  - 跨平台支持 (Windows: `.exe`, Unix: 无后缀)
- [ ] 运行时能力检测
  - TypeScript 原生支持 (Node.js ≥23.6)
  - stdin/stdout 支持验证

**依赖**: 无

**预估时间**: 4-5 小时

### 3.2 进程管理
**位置**: `projects/cli/src/bridge/runtime/process.rs`

**待完成**:
- [ ] RuntimeProcess 结构体
  - 持有 `std::process::Child`
  - stdin/stdout handles
- [ ] 进程启动
  ```rust
  pub fn spawn(runtime: RuntimeType, script_path: &Path, env: HashMap<String, String>) -> Result<Self>
  ```
- [ ] 环境变量设置 (NSAGA_CLI_*)
- [ ] 进程监控和重启
- [ ] 优雅关闭 (SIGTERM)

**依赖**: Runtime Discovery

**预估时间**: 4-5 小时

### 3.3 集成测试
**位置**: `projects/cli/src/bridge/runtime/tests.rs`

**待完成**:
- [ ] 运行时检测测试
- [ ] 进程启动和关闭测试
- [ ] stdin/stdout 通信测试
- [ ] 异常情况处理测试 (进程崩溃、超时)

**依赖**: Runtime Process

**预估时间**: 3-4 小时

### 输出产物
- [ ] `bridge::runtime` 模块
- [ ] 跨平台运行时检测和管理

---

## 阶段 4: Rust Bridge 管理层 🚧 (待实现)

### 4.1 Bridge 实例抽象
**位置**: `projects/cli/src/bridge/manager/bridge.rs`

**待完成**:
- [ ] Bridge trait
  ```rust
  pub trait Bridge {
      fn name(&self) -> &str;
      fn call(&mut self, method: &str, params: Value) -> Result<Value>;
      fn is_healthy(&self) -> bool;
  }
  ```
- [ ] NodeBridge 实现
  - 持有 RuntimeProcess
  - 持有 RpcClient
  - 健康检查

**依赖**: Runtime Process, RPC Client

**预估时间**: 3-4 小时

### 4.2 Bridge Manager 实现
**位置**: `projects/cli/src/bridge/manager/mod.rs`

**待完成**:
- [ ] BridgeManager 结构体
  - 管理多个 Bridge 实例
  - 懒加载 (首次调用时启动)
- [ ] Bridge 注册
  ```rust
  pub fn register(&mut self, name: &str, bridge: Box<dyn Bridge>) -> Result<()>
  ```
- [ ] Bridge 获取和调用
  ```rust
  pub fn call(&mut self, bridge_name: &str, method: &str, params: Value) -> Result<Value>
  ```
- [ ] 生命周期管理
  - 自动启动
  - 失败重试
  - 优雅关闭
- [ ] 线程安全 (Arc<Mutex<>>)

**依赖**: Bridge trait

**预估时间**: 4-5 小时

### 4.3 测试
**位置**: `projects/cli/src/bridge/manager/tests.rs`

**待完成**:
- [ ] 单桥测试
- [ ] 多桥并发测试
- [ ] 生命周期测试
- [ ] 错误恢复测试

**依赖**: Bridge Manager

**预估时间**: 3-4 小时

### 输出产物
- [ ] `bridge::manager` 模块
- [ ] 统一的 Bridge 管理接口

---

## 阶段 5: 配置加载器集成 🚧 (待实现)

### 5.1 ConfigLoader 实现
**位置**: `projects/cli/src/config/loader.rs`

**待完成**:
- [ ] ConfigLoader 结构体
  - 持有 BridgeManager
- [ ] 配置文件发现
  - 文件名匹配 (`novelsaga.config`, `.novelsaga`)
  - 扩展名检测 (`.js`, `.mjs`, `.cjs`, `.ts`, `.mts`, `.cts`)
  - 向上搜索直到 workspace root
- [ ] 配置加载
  ```rust
  pub fn load(&mut self, path: &Path) -> Result<NovelSagaConfig>
  ```
  - 检测文件类型 (CJS/ESM/TS)
  - 设置环境变量
  - 调用 config-bridge
  - 反序列化结果
- [ ] 错误处理
  - 配置文件不存在
  - 配置格式错误
  - Bridge 通信失败

**依赖**: Bridge Manager, Config 类型定义

**预估时间**: 4-5 小时

### 5.2 Initializer 集成
**位置**: `projects/core/src/state/init.rs`

**待完成**:
- [ ] Feature 扩展
  ```rust
  pub struct Feature {
      pub js_loader: Option<Box<dyn Fn(&Path) -> Result<NovelSagaConfig>>>,
      pub ts_loader: Option<Box<dyn Fn(&Path) -> Result<NovelSagaConfig>>>,
  }
  ```
- [ ] Initializer::init() 修改
  - 检查 Feature 中的 loader
  - 优先使用 JS/TS loader (如果提供)
  - 回退到默认 loader (TOML/JSON/RON)
- [ ] CLI 中注册 loader
  ```rust
  let feature = Feature {
      js_loader: Some(Box::new(|path| config_loader.load(path))),
      ts_loader: Some(Box::new(|path| config_loader.load(path))),
  };
  Initializer::init(feature)?;
  ```

**依赖**: ConfigLoader

**预估时间**: 3-4 小时

### 5.3 端到端测试
**位置**: `projects/cli/tests/config_loader_e2e.rs`

**待完成**:
- [ ] 完整配置加载流程测试
  - 创建测试配置文件 (.js, .ts, .cjs, .mjs)
  - 调用 Initializer::init()
  - 验证配置正确加载
- [ ] 配置文件优先级测试
- [ ] 层级配置测试 (RootConfig + OverridableConfig)
- [ ] 错误场景测试

**依赖**: Initializer 集成

**预估时间**: 4-5 小时

### 输出产物
- [ ] `config::loader` 模块
- [ ] 完整的 JS/TS 配置加载功能
- [ ] 集成到 NovelSaga 初始化流程

---

## 阶段 6: 高级特性和优化 🔮 (未来)

### 6.1 Hot-Reload 支持
**待完成**:
- [ ] 文件监控 (notify crate)
- [ ] 配置更新通知
- [ ] 缓存失效策略

**预估时间**: 5-6 小时

### 6.2 性能优化
**待完成**:
- [ ] Bridge 连接池
- [ ] 配置缓存优化
- [ ] 并发加载优化

**预估时间**: 4-5 小时

### 6.3 文档和示例
**待完成**:
- [ ] 用户文档
  - 配置文件编写指南
  - 函数式配置示例
  - 环境变量说明
- [ ] 开发者文档
  - Bridge 开发指南
  - 新运行时适配教程
- [ ] 示例项目

**预估时间**: 6-8 小时

### 输出产物
- [ ] Hot-reload 功能
- [ ] 性能优化
- [ ] 完整文档

---

## 时间估算

| 阶段 | 预估时间 | 状态 |
|------|---------|------|
| 阶段 0: 架构设计 | 已完成 | ✅ |
| 阶段 1: JS Bridge 基础设施 | 已完成 | ✅ |
| 阶段 2: Rust JSON-RPC 通信层 | 14-18h | 🚧 |
| 阶段 3: Rust 运行时管理层 | 11-14h | 🚧 |
| 阶段 4: Rust Bridge 管理层 | 10-13h | 🚧 |
| 阶段 5: 配置加载器集成 | 11-14h | 🚧 |
| 阶段 6: 高级特性和优化 | 15-19h | 🔮 |
| **总计 (阶段 2-5)** | **46-59h** | - |

---

## 关键里程碑

### 里程碑 1: JS 侧完成 ✅
- **日期**: 2026-01-22
- **标志**: config-bridge 测试通过
- **成果**: 可执行的 JS bridge，支持所有配置格式

### 里程碑 2: Rust 通信层完成 🚧
- **预计**: +2 周
- **标志**: Rust 可与 config-bridge 通信
- **成果**: RPC Client 可调用 config.get

### 里程碑 3: 运行时管理完成 🚧
- **预计**: +3 周
- **标志**: 可自动检测和启动 Node.js/Bun/Deno
- **成果**: RuntimeProcess 稳定运行

### 里程碑 4: 配置加载集成完成 🚧
- **预计**: +4 周
- **标志**: `Initializer::init()` 支持 JS/TS 配置
- **成果**: 完整的配置加载功能

### 里程碑 5: 产品化 🔮
- **预计**: +6 周
- **标志**: Hot-reload、文档完善
- **成果**: 生产就绪

---

## 风险和依赖

### 技术风险
1. **跨平台运行时检测复杂性** (中)
   - 缓解: 优先支持标准安装路径，逐步支持版本管理器
2. **进程通信稳定性** (低)
   - 缓解: 完善的错误处理和重试机制
3. **TypeScript 支持的运行时版本差异** (低)
   - 缓解: 检测版本，回退到 JS

### 外部依赖
- Node.js ≥20 (用户环境)
- Rust nightly (开发环境)
- pnpm (构建系统)

---

## 下一步行动

### 立即开始 (阶段 2)
1. 创建 Rust 项目结构
   ```
   projects/cli/src/bridge/
   ├── mod.rs
   ├── rpc/
   │   ├── mod.rs
   │   ├── types.rs
   │   └── client.rs
   └── transport/
       ├── mod.rs
       └── stdio.rs
   ```

2. 实现 JSON-RPC 类型定义
   - 参考 JS 侧的 `bridge-core/src/types/rpc.ts`
   - 使用 `serde_json` 进行序列化

3. 实现 StdioTransport
   - 测试与 config-bridge 的基本通信

### 准备工作
- [ ] 更新 `.github/copilot-instructions.md` 添加 Rust Bridge 开发指南
- [ ] 创建 Rust 测试用例模板
- [ ] 准备 config-bridge 测试脚本

---

**文档版本**: v1.0
**最后更新**: 2026-01-22
**维护者**: AI Agent (GitHub Copilot)
