/**
 * Config Service - 加载和管理配置文件
 */

import type { Service } from "@nsaga/bridge-core/interfaces/service";
import type { NovelSagaConfig } from "../types/config.js";

interface GlobalSettings {
  CONFIG_IS_COMMONJS: boolean;
  CONFIG_IS_TYPESCRIPT: boolean;
  CONFIG_PATH?: string;
  PROJECT_ROOT?: string;
  IS_DEV: boolean;
  VERBOSE_LOG: boolean;
}

/**
 * 配置服务
 */
export class ConfigService implements Service {
  private settings: GlobalSettings;

  constructor() {
    // 从环境变量读取配置
    this.settings = {
      CONFIG_IS_COMMONJS: process.env.NSAGA_CLI_CONFIG_IS_COMMONJS === "true",
      CONFIG_IS_TYPESCRIPT: process.env.NSAGA_CLI_CONFIG_IS_TYPESCRIPT === "true",
      CONFIG_PATH: process.env.NSAGA_CLI_CONFIG_PATH,
      PROJECT_ROOT: process.env.NSAGA_CLI_PROJECT_ROOT,
      IS_DEV: process.env.NSAGA_CLI_IS_DEV === "true",
      VERBOSE_LOG: process.env.NSAGA_CLI_VERBOSE_LOG === "true",
    };
  }

  // 添加索引签名以符合 Service 接口
  [method: string]: any;

  /**
   * 获取配置
   */
  async get(): Promise<NovelSagaConfig> {
    if (!this.settings.CONFIG_PATH) {
      throw new Error("CONFIG_PATH not defined");
    }

    // 动态导入配置文件
    const configModule = await import(this.settings.CONFIG_PATH);

    // 处理 CommonJS 和 ESM 导出
    // Node.js: CJS 通过 import() 会返回 { default: module.exports }
    // Bun: 可能直接返回 module.exports 或 { default: ... }
    let configMain: NovelSagaConfig | ((settings: GlobalSettings) => NovelSagaConfig);

    if (this.settings.CONFIG_IS_COMMONJS) {
      // CJS: 尝试 .default (Node.js) 或直接使用 (Bun)
      configMain = configModule.default || configModule;
    } else {
      // ESM: 使用 .default 或回退到整个模块
      configMain = configModule.default || configModule;
    }

    // 如果是函数，调用它
    let config: NovelSagaConfig;
    if (typeof configMain === "function") {
      config = configMain(this.settings);
    } else {
      config = configMain;
    }

    // 过滤掉所有函数，确保可以序列化
    return this.sanitizeConfig(config);
  }

  /**
   * 净化配置，移除不可序列化的内容
   */
  private sanitizeConfig(obj: unknown): any {
    if (obj === null || obj === undefined) {
      return obj;
    }

    if (typeof obj === "function") {
      return undefined;
    }

    if (Array.isArray(obj)) {
      return obj.map((item) => this.sanitizeConfig(item)).filter((item) => item !== undefined);
    }

    if (typeof obj === "object") {
      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        const sanitized = this.sanitizeConfig(value);
        if (sanitized !== undefined) {
          result[key] = sanitized;
        }
      }
      return result;
    }

    return obj;
  }
}
