// Declare the submodules
pub mod release;
pub mod release_asset;

// Re-export the structs/items you want to be accessible
// directly via `crate::github::models::`
pub use release::Release;
pub use release_asset::ReleaseAsset;
