#[cfg(any(feature = "native", target_arch = "wasm32"))]
mod gpu;

#[cfg(any(feature = "native", target_arch = "wasm32"))]
pub use gpu::Renderer3D;

#[cfg(target_arch = "wasm32")]
pub use gpu::new_wasm_renderer;
