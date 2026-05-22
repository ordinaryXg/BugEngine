use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::game_app::GameApp;
use crate::renderer3d::new_wasm_renderer;

thread_local! {
    static RUNTIME: RefCell<Option<RuntimeState>> = RefCell::new(None);
}

struct RuntimeState {
    renderer: crate::renderer3d::Renderer3D,
    game: GameApp,
    last_time: f64,
}

#[wasm_bindgen]
pub async fn init_runtime(canvas_id: &str, scene_json: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or("canvas not found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "element is not canvas")?;

    let game = GameApp::from_scene_json(scene_json).map_err(|e| e.to_string())?;
    let mut renderer = new_wasm_renderer(canvas).await;
    renderer.upload_meshes(&game.scene.meshes);

    let perf = window.performance().ok_or("no performance")?;
    let last_time = perf.now();

    RUNTIME.with(|cell| {
        *cell.borrow_mut() = Some(RuntimeState {
            renderer,
            game,
            last_time,
        });
    });

    bind_input()?;
    request_frame()?;
    Ok(())
}

fn bind_input() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        let pressed = event.type_() == "keydown";
        RUNTIME.with(|cell| {
            if let Some(state) = cell.borrow_mut().as_mut() {
                match key.as_str() {
                    "w" | "W" => state.game.input.forward = pressed,
                    "s" | "S" => state.game.input.backward = pressed,
                    "a" | "A" => state.game.input.left = pressed,
                    "d" | "D" => state.game.input.right = pressed,
                    _ => {}
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    let callback = closure.as_ref().unchecked_ref();
    window.add_event_listener_with_callback("keydown", callback)?;
    window.add_event_listener_with_callback("keyup", callback)?;
    closure.forget();
    Ok(())
}

fn request_frame() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if let Err(err) = frame_update() {
            web_sys::console::error_1(&err);
        }
        if let Some(win) = web_sys::window() {
            if let Some(cb) = f.borrow().as_ref() {
                let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
            }
        }
    }) as Box<dyn FnMut()>));
    let binding = g.borrow();
    let cb = binding.as_ref().unwrap().as_ref().unchecked_ref();
    window.request_animation_frame(cb)?;
    Ok(())
}

fn frame_update() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let perf = window.performance().ok_or("no performance")?;
    let now = perf.now();

    RUNTIME.with(|cell| {
        if let Some(state) = cell.borrow_mut().as_mut() {
            let dt = ((now - state.last_time) / 1000.0) as f32;
            state.last_time = now;
            state.game.update(dt);
            let _ = state.renderer.render(&state.game);
        }
    });
    Ok(())
}
