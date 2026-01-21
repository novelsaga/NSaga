use std::{collections::HashMap, error::Error, sync::Arc};

use derive_new::new;

pub type LoaderFn =
  Arc<dyn Fn(&str) -> Result<HashMap<String, serde_json::Value>, Box<dyn Error + Send + Sync>> + Send + Sync>;

#[derive(Clone, new)]
pub struct Feature {
  js_loader: Option<LoaderFn>,
  ts_loader: Option<LoaderFn>,
}

impl std::fmt::Debug for Feature {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Feature")
      .field("js_loader", &self.js_loader.as_ref().map(|_| "<function>"))
      .field("ts_loader", &self.ts_loader.as_ref().map(|_| "<function>"))
      .finish()
  }
}

impl Feature {
  #[must_use]
  pub fn js_support(&self) -> bool {
    self.js_loader.is_some()
  }

  #[must_use]
  pub fn ts_support(&self) -> bool {
    self.ts_loader.is_some()
  }

  #[must_use]
  pub fn js_loader(&self) -> Option<&LoaderFn> {
    self.js_loader.as_ref()
  }

  #[must_use]
  pub fn ts_loader(&self) -> Option<&LoaderFn> {
    self.ts_loader.as_ref()
  }
}
