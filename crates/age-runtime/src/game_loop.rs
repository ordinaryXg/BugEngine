#[cfg(feature = "native")]
use std::sync::Arc;
#[cfg(feature = "native")]
use std::time::Instant;

#[cfg(feature = "native")]
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

#[cfg(feature = "native")]
use crate::game_app::GameApp;
#[cfg(feature = "native")]
use crate::renderer3d::Renderer3D;

#[cfg(feature = "native")]
pub struct RuntimeApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer3D>,
    game: GameApp,
    last_frame: Instant,
}

#[cfg(feature = "native")]
impl RuntimeApp {
    pub fn new(game: GameApp) -> Self {
        Self {
            window: None,
            renderer: None,
            game,
            last_frame: Instant::now(),
        }
    }
}

#[cfg(feature = "native")]
impl ApplicationHandler for RuntimeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("AgentGameEngine Runtime")
                            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
                    )
                    .expect("window"),
            );
            let mut renderer = pollster::block_on(Renderer3D::new(window.clone()));
            renderer.upload_meshes(&self.game.scene.meshes);
            self.window = Some(window);
            self.renderer = Some(renderer);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.game.input.forward = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.game.input.backward = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.game.input.left = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.game.input.right = pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right {
                    self.game.input.rmb_down = state == ElementState::Pressed;
                    if let Some(window) = &self.window {
                        if state == ElementState::Pressed {
                            let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                            window.set_cursor_visible(false);
                        } else {
                            let _ = window.set_cursor_grab(CursorGrabMode::None);
                            window.set_cursor_visible(true);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;
                self.game.update(dt);
                if let Some(renderer) = self.renderer.as_mut() {
                    let _ = renderer.render(&self.game);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if self.game.input.rmb_down {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.game.input.mouse_delta.0 += delta.0 as f32;
                self.game.input.mouse_delta.1 += delta.1 as f32;
            }
        }
    }
}

#[cfg(feature = "native")]
pub fn run_native(game: GameApp) {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = RuntimeApp::new(game);
    event_loop.run_app(&mut app).expect("run");
}
