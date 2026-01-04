use std::sync::OnceLock;

use super::{_state::StateBuilder, State, feat::Feature};

/// 全局状态初始化器错误类型
#[derive(Debug)]
pub enum InitError {
  /// 已经初始化过
  AlreadyInitialized,
  /// 尚未初始化就尝试访问
  Uninitialized,
  /// state 构建错误
  StateBuildError,
}

pub struct Initializer;

static GLOBAL_STATE: OnceLock<State> = OnceLock::new();

impl Initializer {
  /// 以给定的 `State` 初始化全局状态，已经初始化则返回 `InitError::AlreadyInitialized`。
  ///
  /// # Errors
  ///
  /// Returns `InitError::AlreadyInitialized` if the global state was already initialized.
  ///
  /// # Panics
  ///
  /// Panics if the `State` builder fails (should not happen with valid inputs).
  pub fn init(feature: Feature) -> Result<&'static State, InitError> {
    // 使用链式 Builder 构造 State，然后放入全局单例
    match StateBuilder::default().feature(feature).build() {
      Ok(st) => {
        GLOBAL_STATE.set(st).map_err(|_| InitError::AlreadyInitialized)?;
        GLOBAL_STATE.get().ok_or(InitError::Uninitialized)
      }
      Err(e) => {
        dbg!(e);
        Err(InitError::StateBuildError)
      }
    }
  }

  /// 如果尚未初始化，使用提供的闭包懒初始化并返回全局状态引用；如果已经初始化则直接返回引用。
  pub fn get_or_init_with(f: impl FnOnce() -> State) -> &'static State {
    GLOBAL_STATE.get_or_init(f)
  }

  /// 尝试获取全局只读访问，若尚未初始化则返回错误。
  ///
  /// # Errors
  ///
  /// Returns `InitError::Uninitialized` if the global state has not been initialized.
  pub fn with_read<R>(f: impl FnOnce(&State) -> R) -> Result<R, InitError> {
    let state = GLOBAL_STATE.get().ok_or(InitError::Uninitialized)?;
    Ok(f(state))
  }

  /// 直接返回全局 `State` 的引用（若未初始化则返回 `Err`）。
  ///
  /// # Errors
  ///
  /// Returns `InitError::Uninitialized` if the global state has not been initialized yet.
  pub fn get() -> Result<&'static State, InitError> {
    GLOBAL_STATE.get().ok_or(InitError::Uninitialized)
  }
}

// 可选：为测试或调试提供内部访问（非公开 API）
// 测试略（依赖于 crate 内部的 `feat::Feature` 类型，实际测试请在同 crate 内编写）
