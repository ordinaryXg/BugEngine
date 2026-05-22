#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub rmb_down: bool,
    pub mouse_delta: (f32, f32),
    pub interact: bool,
}

impl InputState {
    pub fn clear_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.interact = false;
    }
}
