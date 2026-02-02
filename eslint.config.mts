import fs from 'node:fs'
import { builtinModules } from 'node:module'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { FlatCompat } from '@eslint/eslintrc'
import eslint from '@eslint/js'
import { findWorkspacePackages } from '@pnpm/find-workspace-packages'
import EslintConfigPrettier from 'eslint-config-prettier'
// @ts-expect-error -- no-types
import eslintPluginEslintComments from 'eslint-plugin-eslint-comments'
import EslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended'
import { defineConfig } from 'eslint/config'
import globals from 'globals'
import { keys } from 'ramda'
import tseslint from 'typescript-eslint'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const _compat = new FlatCompat({
  baseDirectory: __dirname,
})

const packages = await findWorkspacePackages(__dirname)

const bunPackages = packages.filter((p) => keys(p.manifest.devDependencies || {}).includes('@types/bun'))

const denoPackages = packages.filter((p) => fs.existsSync(path.join(p.dir, 'deno.d.ts')))

const nodejsPackages = packages.filter((p) => !bunPackages.includes(p) && !denoPackages.includes(p))

const commonIgnores = [
  '**/node_modules/**/*',
  '**/dist/**/*',
  '**/build/**/*',
  '**/out/**/*',
  '**/target/**/*',
  '**/deno.d.ts',
]

const getRootAndSrcTS = (root: string) => [`${root}/src/**/*.{ts,cts,mts}`, `${root}/*.{ts,cts,mts}`]

const commonTsFile = packages.flatMap((p) => getRootAndSrcTS(path.relative(__dirname, p.dir) || '.'))

const common = {
  files: commonTsFile,
  ignores: commonIgnores,
}

const tseslintConfig = defineConfig(
  { ...eslint.configs.recommended, ...common },
  ...tseslint.configs.strictTypeChecked.map((config) => ({
    ...config,
    ...common,
  })),
  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-unsafe-member-access -- compact
  ..._compat.config(eslintPluginEslintComments.configs.recommended).map((config) => ({
    ...config,
    ...common,
  })),
  {
    ...EslintPluginPrettierRecommended,
    files: [...(EslintPluginPrettierRecommended.files || []), ...commonTsFile],
    ignores: [...(EslintPluginPrettierRecommended.ignores || []), ...commonIgnores],
  },
  { ...EslintConfigPrettier, ...common },
  ...(
    [
      [bunPackages, globals.node],
      [denoPackages, globals.denoBuiltin],
      [nodejsPackages, globals.bunBuiltin],
    ] as const
  ).map(([p, g]) => ({
    files: p.flatMap((p) => [`${path.relative(__dirname, p.dir) || '.'}/src/**/*.{ts,cts,mts}`]),
    ignores: commonIgnores,
    languageOptions: {
      globals: {
        ...g,
      },
    },
  })),
  ...[bunPackages, denoPackages].map((p) => ({
    files: p.flatMap((p) => [`${path.relative(__dirname, p.dir) || '.'}/*.{ts,cts,mts}`]),
    ignores: commonIgnores,
    languageOptions: {
      globals: {
        ...globals.node,
      },
    },
  })),
  {
    ...common,
    languageOptions: {
      parserOptions: {
        projectService: {
          defaultProject: `${__dirname}/tsconfig.json`,
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/consistent-type-imports': [
        'error',
        {
          prefer: 'type-imports',
          fixStyle: 'inline-type-imports',
        },
      ],
      '@typescript-eslint/no-confusing-void-expression': ['error', { ignoreArrowShorthand: true }],
      '@typescript-eslint/unbound-method': ['error', { ignoreStatic: true }],
      '@typescript-eslint/no-unnecessary-condition': [
        'error',
        {
          allowConstantLoopConditions: true,
        },
      ],
      '@typescript-eslint/no-floating-promises': [
        'error',
        {
          checkThenables: true,
          ignoreVoid: true,
          ignoreIIFE: true,
        },
      ],
      '@typescript-eslint/restrict-template-expressions': [
        'error',
        {
          allowAny: false,
          allowBoolean: true,
          allowArray: false,
          allowNever: false,
          allowNullish: false,
          allowNumber: true,
          allowRegExp: true,
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          caughtErrors: 'all',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
      '@typescript-eslint/no-misused-promises': [
        'error',
        {
          checksConditionals: true,
          checksSpreads: true,
          checksVoidReturn: {
            arguments: false,
            attributes: false,
          },
        },
      ],
      '@typescript-eslint/no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@/swagger/api'],
              message: '使用useAPI',
              allowTypeImports: true,
              importNames: ['default'],
            },
            {
              group: ['@assets/iconify-icons/generated-icons'],
              message: '使用 @assets/iconify-icons/classes',
              allowTypeImports: true,
              importNames: ['default'],
            },
            {
              group: ['enum_overrides'],
              message: '请导入覆盖后的类型',
              allowTypeImports: false,
            },
            {
              group: ['@assets/iconify-icons/classes'],
              message: '此对象太大，请使用宏进行导入',
              allowTypeImports: true,
              importNames: ['default'],
            },
            {
              group: ['___generated___'],
              message: '请勿使用生成的代码',
            },
          ],
          paths: builtinModules.map((name) => ({
            name,
            message: `please import from 'node:${name}' instead`,
          })),
        },
      ],
    },
  },
)

export default tseslintConfig
