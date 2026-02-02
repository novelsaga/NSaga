import { parseTOML } from 'confbox'

export const getVersionFromCargoTomlThisWorkspace = async () => {
  const fs = await import('node:fs/promises')
  const tomlContent = await fs.readFile('../../../Cargo.toml', 'utf-8')
  const parsed: {
    workspace: {
      package: {
        version: string
      }
    }
  } = parseTOML(tomlContent)
  return parsed.workspace.package.version
}
