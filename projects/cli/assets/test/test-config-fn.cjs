// 测试配置文件 - CommonJS 函数形式
module.exports = (settings) => {
  console.error(`[CJS Config Function] Called with settings:`, settings);

  return {
    workspace: {
      cache_dir: `${settings.PROJECT_ROOT}/.novelsaga/cache-cjs`,
      novelsaga_dir: ".novelsaga"
    },
    fmt: {
      indent: settings.IS_DEV ? 4 : 0,
      line_width: 120
    }
  };
};
