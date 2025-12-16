use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
pub struct WorkspaceConfig {}
