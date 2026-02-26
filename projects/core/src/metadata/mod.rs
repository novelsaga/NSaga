//! 元数据管理模块
//!
//! 提供小说元数据的定义、解析和查询接口

pub mod model;
pub mod parser;
pub mod query;

pub use model::MetadataEntity;
pub use parser::{generate_namespace, infer_type_from_path, resolve_type};
pub use query::{MetadataQuery, QueryResult};
