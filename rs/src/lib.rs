use nalgebra::{Matrix3, Vector2};
use wasm_bindgen::prelude::*;

mod golem_backend;
mod schematic;
mod zs_cad;

use golem_backend::ZsCadGolemBackend;
use zs_cad::{Color, DrawList};

#[wasm_bindgen]
pub struct ZsSchIo {
    mouse: Vector2<f32>,
    wheel: Vector2<f32>,
    wheel_pinch: f32,
    keydown: Option<String>,
}

#[wasm_bindgen]
impl ZsSchIo {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            mouse: Vector2::zeros(),
            wheel: Vector2::zeros(),
            wheel_pinch: 0.0,
            keydown: None,
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

    #[wasm_bindgen(setter)]
    pub fn set_keydown(&mut self, key: Option<String>) {
        self.keydown = key;
    }

    pub fn set_wheel_delta(&mut self, x: f32, y: f32, pinch: f32) {
        self.wheel = Vector2::new(x, y);
        self.wheel_pinch = pinch;
    }

    pub fn reset(&mut self) {
        self.wheel = Vector2::zeros();
        self.wheel_pinch = 0.0;
        self.keydown = None;
    }
}

#[wasm_bindgen]
pub struct ZsSch {
    backend: ZsCadGolemBackend,
    draw_list: DrawList,
    pan: Vector2<f32>,
    zoom: f32,
    cursor: Vector2<i32>,
    state: State,
}

const GRID_SIZE: f32 = 16.0;

enum State {
    Ready,
    Wiring(Wiring),
}

enum Wire {
    H(schematic::WireH),
    V(schematic::WireV),
}

struct Wiring {
    segments: Vec<Wire>,
    last: Vector2<i32>,
}

#[wasm_bindgen]
impl ZsSch {
    #[wasm_bindgen(constructor)]
    pub fn new(webgl_context: web_sys::WebGlRenderingContext) -> Self {
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let golem_context = golem::Context::from_glow(glow_context).unwrap();
        let backend = ZsCadGolemBackend::new(golem_context).unwrap();
        let draw_list = DrawList::new(Vector2::new(0, 0));
        Self {
            backend,
            draw_list,
            pan: Vector2::zeros(),
            zoom: 1.,
            cursor: Vector2::zeros(),
            state: State::Ready,
        }
    }

    fn process_pan_zoom(&mut self, io: &ZsSchIo) {
        if io.wheel_pinch != 0.0 {
            let scale = 1. - io.wheel_pinch * 0.02;
            let mouse_mat = Matrix3::new(1., 0., io.mouse.x, 0., 1., io.mouse.y, 0., 0., 1.);
            let scale_mat = Matrix3::new(scale, 0., 0., 0., scale, 0., 0., 0., 1.);
            let mouse_inv_mat =
                Matrix3::new(1., 0., -io.mouse.x, 0., 1., -io.mouse.y, 0., 0., 1.);
            let current_mat = Matrix3::new(
                self.zoom, 0., self.pan.x, 0., self.zoom, self.pan.y, 0., 0., 1.,
            );
            let next_mat = mouse_mat * scale_mat * mouse_inv_mat * current_mat;
            self.zoom = next_mat.m11;
            self.pan = Vector2::new(next_mat.m13, next_mat.m23);
        }
        self.pan -= io.wheel;
    }

    fn process_cursor(&mut self, io: &ZsSchIo) {
        self.cursor = self.screen_to_model(io.mouse).map(|x| x.round() as i32);
    }

    pub fn new_frame(&mut self, io: &mut ZsSchIo) {
        self.process_pan_zoom(&io);
        self.process_cursor(&io);
        if let Some(key) = io.keydown.as_ref() {
            let key = key.as_str();
            match &mut self.state {
                State::Ready => match key {
                    "w" => {
                        self.state = State::Wiring(Wiring {
                            segments: vec![],
                            last: self.cursor,
                        });
                    }
                    _ => {}
                },
                State::Wiring(Wiring { segments, last }) => match key {
                    "w" => {
                        let h = (last.x - self.cursor.x).abs();
                        let v = (last.y - self.cursor.y).abs();
                        if h > v {
                            let wire = Wire::H(schematic::WireH {
                                y: last.y,
                                x1: std::cmp::min(last.x, self.cursor.x),
                                x2: std::cmp::max(last.x, self.cursor.x),
                            });
                            *last = Vector2::new(self.cursor.x, last.y);
                            segments.push(wire);
                        } else {
                            let wire = Wire::V(schematic::WireV {
                                x: last.x,
                                y1: std::cmp::min(last.y, self.cursor.y),
                                y2: std::cmp::max(last.y, self.cursor.y),
                            });
                            *last = Vector2::new(last.x, self.cursor.y);
                            segments.push(wire);
                        }
                    }
                    "Escape" => {
                        self.state = State::Ready;
                    }
                    _ => {}
                },
            }
        }
        io.reset();
        self.draw_list.clear();
        self.draw_grid();
    }

    fn is_in_frame(&self, top_left: Vector2<f32>, bottom_right: Vector2<f32>) -> bool {
        let frame_size: Vector2<f32> = nalgebra::convert(self.draw_list.frame_size);
        let frame_center = frame_size.scale(0.5);
        let target_size = bottom_right - top_left;
        let target_center = (top_left + bottom_right).scale(0.5);
        let diff = (frame_center - target_center).abs();
        let size = frame_size + target_size;

        diff.x <= size.x && diff.y <= size.y
    }

    fn screen_to_model(&self, screen: Vector2<f32>) -> Vector2<f32> {
        (screen - self.pan).unscale(GRID_SIZE * self.zoom)
    }

    fn model_to_screen(&self, model: Vector2<f32>) -> Vector2<f32> {
        model.scale(GRID_SIZE * self.zoom) + self.pan
    }

    fn draw_grid(&mut self) {
        let mut zoom = self.zoom;
        while zoom < 1.0 {
            zoom *= 10.0;
        }
        let grid_unit = GRID_SIZE * zoom;
        let grid_start = self.pan.map(|x| x % grid_unit);
        for y in 0..(self.draw_list.frame_size.y as f32 / grid_unit).floor() as usize {
            for x in 0..(self.draw_list.frame_size.x as f32 / grid_unit).floor() as usize {
                let offset = Vector2::new(x as f32, y as f32).scale(grid_unit);
                let p = grid_start + offset;
                let grayscale = 0.8;
                let col = Color::new(grayscale, grayscale, grayscale, 1.);
                self.draw_list.add_square(p, 2.0, col);
                //self.draw_list.add_circle(p, 2.0, Color::new(0., 0., 0., 1.),2.0);
            }
        }
    }

    fn draw_cursor(&mut self) {
        let p = self.model_to_screen(nalgebra::convert(self.cursor));
        let col = Color::new(0., 0., 0., 1.);
        let half_size = GRID_SIZE * 2.;
        self.draw_list.add_line(
            Vector2::new(p.x - half_size, p.y),
            Vector2::new(p.x + half_size, p.y),
            col,
            1.0,
        );
        self.draw_list.add_line(
            Vector2::new(p.x, p.y - half_size),
            Vector2::new(p.x, p.y + half_size),
            col,
            1.0,
        );
    }

    fn wire_h(&mut self, id: u32, y: i32, x1: i32, x2: i32) -> bool {
        assert!(x1 <= x2);
        let p1 = Vector2::new(x1 as f32, y as f32).scale(GRID_SIZE * self.zoom) + self.pan;
        let p2 = Vector2::new(x2 as f32, y as f32).scale(GRID_SIZE * self.zoom) + self.pan;
        let thickness = 3.0 * self.zoom;
        let half_thickness = Vector2::new(thickness, thickness).scale(0.5);
        let top_left = p1 - half_thickness;
        let bottom_right = p2 + half_thickness;
        if self.is_in_frame(top_left, bottom_right) {
            let col = Color::new(0., 132. / 255., 0., 1.);
            self.draw_list.add_line(p1, p2, col, thickness);
        }
        false
    }

    fn wire_v(&mut self, id: u32, x: i32, y1: i32, y2: i32) -> bool {
        assert!(y1 <= y2);
        let p1 = Vector2::new(x as f32, y1 as f32).scale(GRID_SIZE * self.zoom) + self.pan;
        let p2 = Vector2::new(x as f32, y2 as f32).scale(GRID_SIZE * self.zoom) + self.pan;
        let thickness = 3.0 * self.zoom;
        let half_thickness = Vector2::new(thickness, thickness).scale(0.5);
        let top_left = p1 - half_thickness;
        let bottom_right = p2 + half_thickness;
        if self.is_in_frame(top_left, bottom_right) {
            let col = Color::new(0., 132. / 255., 0., 1.);
            self.draw_list.add_line(p1, p2, col, thickness);
        }
        false
    }

    fn draw_wiring(&mut self, wiring: &Wiring) {
        for segment in &wiring.segments {
            match segment {
                Wire::H(schematic::WireH { y, x1, x2 }) => {
                    self.wire_h(0, *y, *x1, *x2);
                },
                Wire::V(schematic::WireV { x, y1, y2}) => {
                    self.wire_v(0, *x, *y1, *y2);
                },
            }
        }
        let last = wiring.last;
        let h = (last.x - self.cursor.x).abs();
        let v = (last.y - self.cursor.y).abs();
        if h > v {
            let y = last.y;
            let x1 = std::cmp::min(last.x, self.cursor.x);
            let x2 = std::cmp::max(last.x, self.cursor.x);
            self.wire_h(0, y, x1, x2);
        } else {
            let x = last.x;
            let y1 = std::cmp::min(last.y, self.cursor.y);
            let y2 = std::cmp::max(last.y, self.cursor.y);
            self.wire_v(0, x, y1, y2);
        }
        self.draw_cursor();
    }

    pub fn draw(&mut self) {
        let state = std::mem::replace(&mut self.state, State::Ready);
        match &state {
            State::Wiring(wiring) => {
                self.draw_wiring(&wiring);
            },
            State::Ready => {},
        }
        self.state = state;
        self.backend.draw(&self.draw_list).unwrap();
    }

    pub fn set_frame_size(&mut self, w: u32, h: u32) {
        self.draw_list.frame_size = Vector2::new(w, h);
    }
}
