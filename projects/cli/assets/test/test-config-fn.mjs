// 测试配置文件 - 函数形式
export default (settings) => {
  console.error(`[Config Function] Called with settings:`, settings);
  
  return {
    workspace: {
      cache_dir: `${settings.PROJECT_ROOT}/.novelsaga/cache`,
      novelsaga_dir: ".novelsaga"
    },
    fmt: {
      indent: settings.IS_DEV ? 2 : 0,
      line_width: 100
    }
  };
};
