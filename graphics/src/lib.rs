#![warn(clippy::all)]

pub use crate::common::*;
pub use crate::fonts::*;
pub use crate::resources::*;

pub mod api {
    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))]
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}

pub mod common;
pub mod fonts;
pub mod resources;
mod voxels;
