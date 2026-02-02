//! Environment variable key constants for bridge communication
//!
//! These constants define the environment variable keys used to pass configuration
//! and runtime information from Rust CLI to JavaScript/TypeScript bridges.

/// Path to the configuration file to load
pub const CONFIG_PATH: &str = "NSAGA_CLI_CONFIG_PATH";

/// Project root directory path
pub const PROJECT_ROOT: &str = "NSAGA_CLI_PROJECT_ROOT";

/// Whether the config file is TypeScript (true/false)
pub const CONFIG_IS_TYPESCRIPT: &str = "NSAGA_CLI_CONFIG_IS_TYPESCRIPT";

/// Whether the config file is CommonJS (true/false)
pub const CONFIG_IS_COMMONJS: &str = "NSAGA_CLI_CONFIG_IS_COMMONJS";

/// Whether running in development mode (true/false)
pub const IS_DEV: &str = "NSAGA_CLI_IS_DEV";

/// Whether to enable verbose logging (true/false)
pub const VERBOSE_LOG: &str = "NSAGA_CLI_VERBOSE_LOG";

/// JavaScript runtime type (node/bun/deno)
pub const RUNTIME: &str = "NSAGA_CLI_RUNTIME";
