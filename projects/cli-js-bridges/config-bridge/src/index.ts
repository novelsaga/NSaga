/**
 * Config Bridge - 配置文件加载服务
 * 支持 Node.js, Bun, Deno 多 runtime
 */

import { getVersionFromCargoTomlThisWorkspace } from './macros' with { type: 'macro' }
import { ConfigService } from './services/config.js'

/**
 * 根据环境变量动态加载对应的 runtime adapter
 */
async function loadBridgeFactory() {
  const runtime = process.env.NSAGA_RUNTIME || 'node'

  switch (runtime) {
    case 'bun':
      return await import('@nsaga/bridge-bun')
    case 'deno':
      return await import('@nsaga/bridge-deno')
    case 'node':
    default:
      return await import('@nsaga/bridge-nodejs')
  }
}

async function main() {
  // 动态加载对应 runtime 的工厂函数
  const { createBridgeServer } = await loadBridgeFactory()

  // 使用工厂函数创建 Bridge Server
  const server = createBridgeServer({
    name: 'config-bridge',
    version: await getVersionFromCargoTomlThisWorkspace(),
  })

  // 注册配置服务
  const configService = new ConfigService()
  server.registerService('config', configService)

  // 启动服务器
  server.start()
}

// 启动
main().catch((error: unknown) => {
  console.error('Fatal error:', error)
  process.exit(1)
})
