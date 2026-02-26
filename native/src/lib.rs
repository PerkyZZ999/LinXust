use napi_derive::napi;

#[napi]
pub fn hello_from_rust(name: String) -> String {
  format!("Hello, {name}, from Rust!")
}
