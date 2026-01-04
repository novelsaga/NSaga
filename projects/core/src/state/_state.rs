use derive_builder::Builder;

use super::{feat::Feature, manager::config::ConfigManager};

// 使用 `static_assertions` 在编译期断言 `State: Send + Sync`
static_assertions::assert_impl_all!(State: Send, Sync);

#[derive(Builder, getset::CopyGetters, Debug, getset::Getters)]
#[builder(setter(into), build_fn(skip))]
#[allow(dead_code)]
pub struct State {
  #[getset(get_copy = "pub")]
  feature: Feature,
  #[builder(private)]
  #[getset(get = "pub")]
  config_manager: ConfigManager,
}

impl StateBuilder {
  pub fn build(&self) -> Result<State, StateBuilderError> {
    let feature = self
      .feature
      .ok_or_else(|| StateBuilderError::ValidationError("wrong feature".to_string()))?;

    let config_manager = ConfigManager::new(feature);

    Ok(State {
      feature,
      config_manager,
    })
  }
}
