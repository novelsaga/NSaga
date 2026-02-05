import type { PrettierConfig as PrettierPluginSortImportsConfig } from '@ianvs/prettier-plugin-sort-imports'
import type { Config } from 'prettier'
import type { Options as PrettierPluginJsdocOptions } from 'prettier-plugin-jsdoc'

import { findWorkspaceDir } from '@pnpm/find-workspace-dir'
import { getLockfileImporterId, readWantedLockfile } from '@pnpm/lockfile-file'
import * as prettierPluginOxc from '@prettier/plugin-oxc'
import * as prettierPluginJsdoc from 'prettier-plugin-jsdoc'
import { parse } from 'semver'

const workspaceRoot = await findWorkspaceDir(import.meta.dirname)

if (!workspaceRoot) {
  throw new Error('Could not find workspace root')
}

const lockfile = await readWantedLockfile(workspaceRoot, {
  ignoreIncompatible: false,
})

if (!lockfile?.importers) {
  throw new Error('Could not read lockfile')
}

const importerId = getLockfileImporterId(workspaceRoot, import.meta.dirname)
const rootImporter = lockfile.importers[importerId]
const rawVersion = rootImporter.devDependencies?.['typescript'] ?? '5.0.0'
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
const parsed = parse(rawVersion)
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-explicit-any
const typescriptVer = parsed ? `${(parsed as any).major}.${(parsed as any).minor}.${(parsed as any).patch}` : '5.0.0'

const jsCommon: PrettierPluginSortImportsConfig & PrettierPluginJsdocOptions = {
  singleQuote: true,
  quoteProps: 'as-needed',
  jsxSingleQuote: true,
  trailingComma: 'all',
  bracketSpacing: true,
  bracketSameLine: false,
  arrowParens: 'always',
  proseWrap: 'preserve',
  embeddedLanguageFormatting: 'auto',
  /* cspell:disable-next-line */
  plugins: [
    prettierPluginOxc,
    prettierPluginJsdoc as Omit<typeof prettierPluginJsdoc, 'defaultOptions'>,
    '@ianvs/prettier-plugin-sort-imports',
  ],
  importOrder: [
    '',
    '<TYPES>^(node:)',
    '',
    '<TYPES>',
    '',
    // ...pathAlias.map((a) => [`<TYPES>^${a}/(.*)$`, '']).flat(),
    '<TYPES>^[./]',
    '',
    '<BUILTIN_MODULES>',
    '',
    '<THIRD_PARTY_MODULES>',
    '',
    // ...pathAlias.map((a) => [`^${a}/(.*)$`, '']).flat(),
    '^[./]',
    '^(?!.*[.]css$)[./].*$',
    '.css$',
  ],
  importOrderParserPlugins: [
    'typescript',
    'jsx',
    'classProperties',
    'decorators',
    'dynamicImport',
    '["importAttributes", { "deprecatedAssertSyntax": true }]',
  ],
  importOrderTypeScriptVersion: typescriptVer,
  jsdocSeparateReturnsFromParam: true,
  jsdocSeparateTagGroups: true,
  jsdocPreferCodeFences: true,
}

const config: Config = {
  printWidth: 120,
  tabWidth: 2,
  useTabs: false,
  semi: false,
  singleQuote: true,
  requirePragma: false,
  insertPragma: false,
  proseWrap: 'preserve',
  htmlWhitespaceSensitivity: 'strict',
  endOfLine: 'auto',
  overrides: [
    {
      files: ['*.ts', '*.mts', '*.cts', '*.tsx'],
      options: jsCommon,
    },
  ],
}

export default config
