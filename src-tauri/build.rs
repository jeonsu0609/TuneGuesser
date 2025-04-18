use std::env;
use std::path::PathBuf;
use tauri_build::{try_build, Attributes, WindowsAttributes};

extern crate bindgen;

fn main() {
  if let Err(error) = try_build(
    Attributes::new()
      .windows_attributes(WindowsAttributes::new().window_icon_path("../.icons/icon.ico")),
  ) {
    panic!("error found during tauri-build: {}", error);
  }
}
