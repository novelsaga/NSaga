/**
 * Service 抽象接口 - 定义 RPC 服务的方法
 */

/**
 * Service 接口
 *
 * 服务类必须实现索引签名，允许动态方法调用
 */
export interface Service {
  [method: string]: any;
}
