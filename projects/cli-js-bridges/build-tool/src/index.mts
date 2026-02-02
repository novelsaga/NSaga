#!/usr/bin/env node
/**
 * Unified build tool for all bridges
 */
import { exec } from 'node:child_process'
import { mkdir, readdir } from 'node:fs/promises'
import { resolve } from 'node:path'
import { promisify } from 'node:util'

import * as esbuild from 'esbuild'
import Macros from 'unplugin-macros/esbuild'

const execAsync = promisify(exec)

export interface BuildOptions {
  /** Bridge 名称（用于日志输出） */
  name: string
  /** 项目根目录（__dirname） */
  rootDir: string
  /** 入口文件（相对于 rootDir，默认自动扫描 src/） */
  entryPoints?: string[]
  /** 是否自动扫描 src/ 目录的所有 .ts 文件（默认 false） */
  autoScanSrc?: boolean
  /** 构建前的钩子函数 */
  preBuild?: () => Promise<void>
  /** 构建后的钩子函数 */
  postBuild?: () => Promise<void>
  /** esbuild 配置覆盖 */
  esbuildOverrides?: Partial<esbuild.BuildOptions>
  /** 是否生成 .d.ts 文件（默认 true） */
  generateDts?: boolean
}

/**
 * 统一的构建函数
 */
export async function buildBridge(options: BuildOptions): Promise<void> {
  const {
    name,
    rootDir,
    entryPoints: userEntryPoints,
    autoScanSrc = false,
    preBuild,
    postBuild,
    esbuildOverrides = {},
    generateDts = true,
  } = options

  const startTime = Date.now()

  try {
    console.log(`🔨 Building ${name}...`)

    // 执行 preBuild 钩子
    if (preBuild) {
      await preBuild()
    }

    // 确定入口文件
    let entryPoints: string[]
    if (userEntryPoints) {
      entryPoints = userEntryPoints.map((p) => resolve(rootDir, p))
    } else if (autoScanSrc) {
      entryPoints = await scanSrcDirectory(rootDir)
    } else {
      // 默认入口点
      entryPoints = [resolve(rootDir, 'src/transport.ts'), resolve(rootDir, 'src/index.ts')].filter((p) => {
        try {
          require.resolve(p)
          return true
        } catch {
          return false
        }
      })
    }

    if (entryPoints.length === 0) {
      throw new Error('No entry points found. Specify entryPoints or enable autoScanSrc.')
    }

    // 确保输出目录存在
    const outdir = resolve(rootDir, 'dist')
    await mkdir(outdir, { recursive: true })

    // 默认 esbuild 配置
    const defaultEsbuildConfig: esbuild.BuildOptions = {
      plugins: [Macros()],
      entryPoints,
      outdir: 'dist',
      outbase: 'src',
      bundle: false, // 不打包，保持模块结构
      platform: 'node',
      target: 'node20',
      format: 'esm',
      sourcemap: false,
      minify: false,
      logLevel: 'warning',
      loader: {
        '.ts': 'ts',
      },
    }

    // 合并用户配置
    const esbuildConfig = {
      ...defaultEsbuildConfig,
      ...esbuildOverrides,
    }

    // 1. 使用 esbuild 编译 JS
    await esbuild.build(esbuildConfig)

    // 2. 生成 .d.ts 文件
    if (generateDts) {
      console.log('📝 Generating .d.ts files...')
      await execAsync('npx tsc --emitDeclarationOnly', { cwd: rootDir })
    }

    // 执行 postBuild 钩子
    if (postBuild) {
      await postBuild()
    }

    const duration = Date.now() - startTime
    console.log(`✅ ${name} build complete! (${duration}ms)`)
    console.log(`   📦 Output: ${outdir}`)
  } catch (error) {
    console.error(`❌ ${name} build failed:`, error)
    process.exit(1)
  }
}

/**
 * 递归扫描 src/ 目录下的所有 .ts 文件
 */
async function scanSrcDirectory(rootDir: string): Promise<string[]> {
  const srcDir = resolve(rootDir, 'src')
  const entryPoints: string[] = []

  async function scanDir(dir: string): Promise<void> {
    const entries = await readdir(dir, { withFileTypes: true })

    for (const entry of entries) {
      const fullPath = resolve(dir, entry.name)
      if (entry.isDirectory()) {
        await scanDir(fullPath)
      } else if (entry.isFile() && entry.name.endsWith('.ts') && !entry.name.endsWith('.d.ts')) {
        entryPoints.push(fullPath)
      }
    }
  }

  await scanDir(srcDir)
  return entryPoints
}
