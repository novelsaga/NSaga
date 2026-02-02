/**
 * Environment variable key constants for bridge communication
 *
 * These constants must match the Rust side definitions in `bridge/env_keys.rs`
 */

/** Path to the configuration file to load */
export const CONFIG_PATH = 'NSAGA_CLI_CONFIG_PATH'

/** Project root directory path */
export const PROJECT_ROOT = 'NSAGA_CLI_PROJECT_ROOT'

/** Whether the config file is TypeScript (true/false) */
export const CONFIG_IS_TYPESCRIPT = 'NSAGA_CLI_CONFIG_IS_TYPESCRIPT'

/** Whether the config file is CommonJS (true/false) */
export const CONFIG_IS_COMMONJS = 'NSAGA_CLI_CONFIG_IS_COMMONJS'

/** Whether running in development mode (true/false) */
export const IS_DEV = 'NSAGA_CLI_IS_DEV'

/** Whether to enable verbose logging (true/false) */
export const VERBOSE_LOG = 'NSAGA_CLI_VERBOSE_LOG'

/** JavaScript runtime type (node/bun/deno) */
export const RUNTIME = 'NSAGA_CLI_RUNTIME'
