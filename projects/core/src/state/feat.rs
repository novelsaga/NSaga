use derive_new::new;

#[derive(getset::CopyGetters, Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct Feature {
  #[getset(get_copy = "pub")]
  ts_support: bool,

  #[getset(get_copy = "pub")]
  js_support: bool,
}
