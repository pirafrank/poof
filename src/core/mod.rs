pub mod musl;
pub mod platform_info;
/// Asset-selection logic: scores release assets by platform compatibility.
pub mod selector;

#[cfg(test)]
mod tests;
