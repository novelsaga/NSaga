/**
 * Config Service - 加载和管理配置文件
 */

import type { Service, ServiceMethod } from '@nsaga/bridge-core/interfaces/service'

import type { NovelSagaConfig } from '../types/config.js'

import * as ENV_KEYS from '../constants/env-keys.js'

interface GlobalSettings {
  CONFIG_IS_COMMONJS: boolean
  CONFIG_IS_TYPESCRIPT: boolean
  CONFIG_PATH?: string
  PROJECT_ROOT?: string
  IS_DEV: boolean
  VERBOSE_LOG: boolean
}

type ConfigFactory = (settings: GlobalSettings) => NovelSagaConfig

interface ConfigModule {
  default?: NovelSagaConfig | ConfigFactory
  [key: string]: unknown
}

export class ConfigService implements Service {
  readonly #settings: GlobalSettings;

  [method: string]: ServiceMethod | undefined

  constructor() {
    this.#settings = {
      CONFIG_IS_COMMONJS: process.env[ENV_KEYS.CONFIG_IS_COMMONJS] === 'true',
      CONFIG_IS_TYPESCRIPT: process.env[ENV_KEYS.CONFIG_IS_TYPESCRIPT] === 'true',
      CONFIG_PATH: process.env[ENV_KEYS.CONFIG_PATH],
      PROJECT_ROOT: process.env[ENV_KEYS.PROJECT_ROOT],
      IS_DEV: process.env[ENV_KEYS.IS_DEV] === 'true',
      VERBOSE_LOG: process.env[ENV_KEYS.VERBOSE_LOG] === 'true',
    }
  }

  async get(): Promise<NovelSagaConfig> {
    if (!this.#settings.CONFIG_PATH) {
      throw new Error('CONFIG_PATH not defined')
    }

    const configModule = (await import(this.#settings.CONFIG_PATH)) as ConfigModule

    let configMain: NovelSagaConfig | ConfigFactory

    if (this.#settings.CONFIG_IS_COMMONJS) {
      configMain = (configModule.default ?? configModule) as NovelSagaConfig | ConfigFactory
    } else {
      configMain = (configModule.default ?? configModule) as NovelSagaConfig | ConfigFactory
    }

    let config: NovelSagaConfig
    if (typeof configMain === 'function') {
      config = configMain(this.#settings)
    } else {
      config = configMain
    }

    return this.sanitizeConfig(config) as NovelSagaConfig
  }

  private sanitizeConfig(obj: unknown): unknown {
    if (obj === null || obj === undefined) {
      return obj
    }

    if (typeof obj === 'function') {
      return undefined
    }

    if (Array.isArray(obj)) {
      return obj.map((item: unknown) => this.sanitizeConfig(item)).filter((item) => item !== undefined)
    }

    if (typeof obj === 'object') {
      const result: Record<string, unknown> = {}
      for (const [key, value] of Object.entries(obj)) {
        const sanitized = this.sanitizeConfig(value)
        if (sanitized !== undefined) {
          result[key] = sanitized
        }
      }
      return result
    }

    return obj
  }
}
