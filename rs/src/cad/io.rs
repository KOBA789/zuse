use nalgebra::Vector2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Io {
    pub(crate) mouse: Vector2<f32>,
    pub(crate) wheel: Vector2<f32>,
    pub(crate) wheel_pinch: f32,
    pub(crate) events: Vec<Event>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Keydown(String),
    Keyup(String),
    MouseDown(u8),
    MouseUp(u8),
    Click(u8),
    DoubleClick(u8),
}

#[wasm_bindgen]
impl Io {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            mouse: Vector2::zeros(),
            wheel: Vector2::zeros(),
            wheel_pinch: 0.0,
            events: vec![],
        }
    }

    #[wasm_bindgen(getter = wheelX)]
    pub fn wheel_x(&self) -> f32 {
        self.wheel.x
    }

    #[wasm_bindgen(setter = wheelX)]
    pub fn set_wheel_x(&mut self, wheel_x: f32) {
        self.wheel.x = wheel_x;
    }

    #[wasm_bindgen(getter = wheelY)]
    pub fn wheel_y(&self) -> f32 {
        self.wheel.y
    }

    #[wasm_bindgen(setter = wheelY)]
    pub fn set_wheel_y(&mut self, wheel_y: f32) {
        self.wheel.y = wheel_y;
    }

    #[wasm_bindgen(getter)]
    pub fn pinch(&self) -> f32 {
        self.wheel.x
    }

    #[wasm_bindgen(setter)]
    pub fn set_pinch(&mut self, pinch: f32) {
        self.wheel_pinch = pinch;
    }

    #[wasm_bindgen(getter = mouseX)]
    pub fn mouse_x(&self) -> f32 {
        self.mouse.x
    }

    #[wasm_bindgen(setter = mouseX)]
    pub fn set_mouse_x(&mut self, mouse_x: f32) {
        self.mouse.x = mouse_x;
    }

    #[wasm_bindgen(getter = mouseY)]
    pub fn mouse_y(&self) -> f32 {
        self.mouse.y
    }

    #[wasm_bindgen(setter = mouseY)]
    pub fn set_mouse_y(&mut self, mouse_y: f32) {
        self.mouse.y = mouse_y;
    }

    #[wasm_bindgen(js_name = pushKeydown)]
    pub fn push_keydown(&mut self, key: String) {
        self.events.push(Event::Keydown(key));
    }

    #[wasm_bindgen(js_name = pushKeyup)]
    pub fn push_keyup(&mut self, key: String) {
        self.events.push(Event::Keyup(key));
    }

    #[wasm_bindgen(js_name = pushClick)]
    pub fn push_click(&mut self, button: u8) {
        self.events.push(Event::Click(button));
    }

    #[wasm_bindgen(js_name = pushDoubleClick)]
    pub fn push_double_click(&mut self, button: u8) {
        self.events.push(Event::DoubleClick(button));
    }

    pub fn set_wheel_delta(&mut self, x: f32, y: f32, pinch: f32) {
        self.wheel = Vector2::new(x, y);
        self.wheel_pinch = pinch;
    }

    pub fn reset(&mut self) {
        self.wheel = Vector2::zeros();
        self.wheel_pinch = 0.0;
        self.events.clear();
    }
}
