/// GitHub release model.
pub mod release;
/// GitHub release asset model.
pub mod release_asset;

// Re-export the structs/items you want to be accessible
// directly via `crate::github::models::`
pub use release::Release;
pub use release_asset::ReleaseAsset;
