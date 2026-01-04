#[cfg(feature = "headers")]
use novelsaga_core::ffi::generate_headers;

#[cfg(feature = "headers")]
fn main() -> std::io::Result<()> {
  generate_headers()
}

#[cfg(not(feature = "headers"))]
fn main() {
  eprintln!("Error: headers feature not enabled");
  std::process::exit(1);
}
