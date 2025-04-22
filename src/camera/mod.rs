#[cfg(feature = "2d")]
mod controller2d;
#[cfg(feature = "3d")]
mod controller3d;

mod plugin;

#[cfg(feature = "2d")]
pub use controller2d::{CameraController2d, CameraView2d};
#[cfg(feature = "3d")]
pub use controller3d::{CameraController3d, CameraView3d};

pub use plugin::CameraPlugin;
