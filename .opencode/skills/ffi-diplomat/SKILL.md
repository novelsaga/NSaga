---
name: ffi-diplomat
description: FFI bindings using Diplomat - use when modifying diplomat_ffi.rs or exposing Rust types to C/WASM
---

# Diplomat FFI Bindings

## Quick Reference

- **Single bridge module**: All FFI in one `#[diplomat::bridge]` module
- **Opaque types**: `#[diplomat::opaque]` for complex Rust types
- **Constructor attr**: `#[diplomat::attr(auto, constructor)]`
- **String handling**: Use `DiplomatStr` (bytes) + `DiplomatWrite` (output)

## Bridge Module

```rust
// All FFI definitions MUST be in a single bridge module
#[diplomat::bridge]
pub mod ffi {
  use diplomat_runtime::{DiplomatStr, DiplomatWrite};

  // Opaque types and impl blocks go here
}
```

## Opaque Types

```rust
// Wrap internal types with #[diplomat::opaque]
#[diplomat::opaque]
pub struct Article(crate::article::Article);

#[diplomat::opaque]
pub struct FormatConfig(crate::config::formatter::FormatConfig);
```

## Method Exposure

```rust
impl Article {
  // Mark constructor with diplomat attribute
  #[diplomat::attr(auto, constructor)]
  pub fn new(content: &DiplomatStr) -> Box<Article> {
    let content_str = core::str::from_utf8(content).unwrap_or("");
    Box::new(Article(crate::article::Article::new(content_str)))
  }

  // Regular methods
  pub fn len(&self) -> usize {
    self.0.content_ref().len()
  }

  // String output uses DiplomatWrite
  pub fn get_content(&self, write: &mut DiplomatWrite) {
    write.write_str(self.0.content_ref()).unwrap();
    write.flush();
  }

  // Return opaque types in Box
  pub fn format(&self, config: &FormatConfig) -> Box<Article> {
    let formatted = crate::library::formatter::format_text(&self.0, &config.0);
    Box::new(Article(formatted))
  }
}
```

## Anti-Patterns

| Don't                                       | Do Instead                             |
| ------------------------------------------- | -------------------------------------- |
| Multiple `#[diplomat::bridge]` modules      | Single module with all FFI definitions |
| Expose internal types directly              | Wrap with `#[diplomat::opaque]`        |
| Return `String` directly                    | Use `&mut DiplomatWrite` for output    |
| Return `&str` from methods                  | Write to `DiplomatWrite` buffer        |
| Forget `Box<T>` for opaque returns          | Always `Box::new()` opaque types       |
| Use `std::string::String` in FFI signatures | Use `DiplomatStr` for input bytes      |

## When to Use

Load this skill when working on:

- `projects/core/src/diplomat_ffi.rs`
- Exposing Rust types to C/C++/WASM
- Adding new FFI methods or types
- Understanding Diplomat patterns and constraints
