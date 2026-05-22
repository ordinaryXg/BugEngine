#[cfg(feature = "native")]
pub mod game_loop;

pub mod game_app;
pub mod input;
pub mod mesh_builtin;
pub mod physics_simple;
pub mod player_controller;
pub mod renderer3d;
pub mod scene_loader;
pub mod vertex;

pub use game_app::GameApp;
pub use scene_loader::{load_scene, LoadedScene};

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::init_runtime;
