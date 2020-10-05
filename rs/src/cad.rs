mod draw_list;
mod io;

use nalgebra::{Matrix3, Vector2};
use wasm_bindgen::prelude::*;
use super::schematic;
use super::backend::GolemBackend;
pub use draw_list::{Color, DrawList, DrawCmd};
pub use io::Io;

#[wasm_bindgen]
pub struct Cad {
    backend: GolemBackend,
    draw_list: DrawList,
    pan: Vector2<f32>,
    zoom: f32,
    cursor: Vector2<i32>,
    tool_state: ToolState,
    sch_state: schematic::State,
}

const GRID_SIZE: f32 = 16.0;

enum ToolState {
    Selection,
    ReadyToWire,
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

impl Wiring {
    fn start(cursor: Vector2<i32>) -> Wiring {
        Wiring {
            segments: vec![],
            last: cursor,
        }
    }

    fn add_segment(&mut self, cursor: Vector2<i32>) {
        let h = (self.last.x - cursor.x).abs();
        let v = (self.last.y - cursor.y).abs();
        match (self.segments.last(), h > v) {
            (Some(Wire::V(_)), _) | (None, true) => {
                let (x1, x2) = ord(self.last.x, cursor.x);
                let wire = Wire::H(schematic::WireH {
                    y: self.last.y,
                    x1,
                    x2,
                });
                self.last = Vector2::new(cursor.x, self.last.y);
                self.segments.push(wire);
            },
            (Some(Wire::H(_)), _) | (None, false) => {
                let (y1, y2) = ord(self.last.y, cursor.y);
                let wire = Wire::V(schematic::WireV {
                    x: self.last.x,
                    y1,
                    y2,
                });
                self.last = Vector2::new(self.last.x, cursor.y);
                self.segments.push(wire);
            },
        }
    }
}

#[inline]
fn ord(v1: i32, v2: i32) -> (i32, i32) {
    if v1 <= v2 {
        (v1, v2)
    } else {
        (v2, v1)
    }
}

#[wasm_bindgen]
impl Cad {
    #[wasm_bindgen(constructor)]
    pub fn new(backend: GolemBackend) -> Self {
        let draw_list = DrawList::new(Vector2::new(0, 0));
        Self {
            backend,
            draw_list,
            pan: Vector2::zeros(),
            zoom: 1.,
            cursor: Vector2::zeros(),
            tool_state: ToolState::Selection,
            sch_state: schematic::State::default(),
        }
    }

    fn process_pan_zoom(&mut self, io: &Io) {
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

    fn process_cursor(&mut self, io: &Io) {
        self.cursor = self.screen_to_model(io.mouse).map(|x| x.round() as i32);
    }

    fn process_event(&mut self, event: &io::Event) {
        match &mut self.tool_state {
            ToolState::Selection => match event {
                io::Event::Keydown(key) if key == "w" => {
                    self.tool_state = ToolState::Wiring(Wiring::start(self.cursor));
                },
                io::Event::Keydown(key) if key == "d" => {
                    self.sch_state.delete_at_point([self.cursor.x, self.cursor.y]);
                }
                _ => {},
            },
            ToolState::ReadyToWire => match event {
                io::Event::Click(0) => {
                    self.tool_state = ToolState::Wiring(Wiring::start(self.cursor));
                },
                io::Event::Keydown(key) if key == "Escape" => {
                    self.tool_state = ToolState::Selection;
                }
                _ => {},
            },
            ToolState::Wiring(wiring) => match event {
                io::Event::Keydown(key) if key == "w" => {
                    wiring.add_segment(self.cursor);
                }
                io::Event::Click(0) => {
                    wiring.add_segment(self.cursor);
                },
                io::Event::DoubleClick(0) => {
                    for wire in &wiring.segments {
                        match wire {
                            Wire::H(wire_h) => {
                                self.sch_state.add_wire(wire_h.clone());
                            },
                            Wire::V(wire_v) => {
                                self.sch_state.add_wire(wire_v.clone());
                            },
                        }
                    }
                    self.tool_state = ToolState::ReadyToWire;
                },
                io::Event::Keydown(key) if key == "Escape" => {
                    self.tool_state = ToolState::Selection;
                }
                _ => {},
            },
        }
    }

    fn process_events(&mut self, io: &Io) {
        for event in &io.events {
            self.process_event(&event);
        }
    }

    pub fn new_frame(&mut self, io: &mut Io) {
        self.process_pan_zoom(&io);
        self.process_cursor(&io);
        self.process_events(&io);
        io.reset();
        self.draw_list.clear();
        self.draw_grid();
    }

    fn is_in_screen(&self, top_left: Vector2<f32>, bottom_right: Vector2<f32>) -> bool {
        let frame_size: Vector2<f32> = nalgebra::convert(self.draw_list.frame_size);
        let frame_center = frame_size.scale(0.5);
        let target_size = bottom_right - top_left;
        let target_center = (top_left + bottom_right).scale(0.5);
        let diff = (frame_center - target_center).abs();
        let size = frame_size + target_size;

        diff.x <= size.x && diff.y <= size.y
    }

    fn model_bound(&self) -> (Vector2<i32>, Vector2<i32>) {
        let top_left = self.screen_to_model(Vector2::zeros()).map(|f| f.floor() as i32);
        let frame_size: Vector2<f32> = nalgebra::convert(self.draw_list.frame_size);
        let bottom_right = self.screen_to_model(frame_size).map(|f| f.ceil() as i32);
        (top_left, bottom_right)
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

    fn draw_schematic(&mut self) {
        let sch_state = std::mem::take(&mut self.sch_state);
        let (top_left, right_bottom) = self.model_bound();
        let aabb = rstar::AABB::from_corners(top_left.into(), right_bottom.into());
        for wire in sch_state.wires_iter(&aabb) {
            self.wire(Vector2::new(wire.from[0], wire.from[1]), Vector2::new(wire.to[0], wire.to[1]));
        }
        for junction in sch_state.junctions_iter(&aabb) {
            self.junction(Vector2::new(junction[0], junction[1]));
        }
        self.sch_state = sch_state;
    }

    fn wire(&mut self, s1: Vector2<i32>, s2: Vector2<i32>) {
        let p1 = nalgebra::convert::<_, Vector2<f32>>(s1).scale(GRID_SIZE * self.zoom) + self.pan;
        let p2 = nalgebra::convert::<_, Vector2<f32>>(s2).scale(GRID_SIZE * self.zoom) + self.pan;
        let thickness = (3.0 * self.zoom).max(1.0);
        let half_thickness = Vector2::new(thickness, thickness).scale(0.5);
        let top_left = p1 - half_thickness;
        let bottom_right = p2 + half_thickness;
        let col = Color::new(0., 132. / 255., 0., 1.);
        self.draw_list.add_line(p1, p2, col, thickness);
    }

    fn junction(&mut self, s: Vector2<i32>) {
        let p = nalgebra::convert::<_, Vector2<f32>>(s).scale(GRID_SIZE * self.zoom) + self.pan;
        let thickness = 12.0 * self.zoom;
        let half_thickness = Vector2::new(thickness, thickness).scale(0.5);
        let top_left = p - half_thickness;
        let bottom_right = p + half_thickness;
        let col = Color::new(0., 132. / 255., 0., 1.);
        self.draw_list.add_line(p, p, col, thickness);
    }

    fn draw_wiring(&mut self, wiring: &Wiring) {
        for segment in &wiring.segments {
            match segment {
                Wire::H(schematic::WireH { y, x1, x2 }) => {
                    self.wire(Vector2::new(*x1, *y), Vector2::new(*x2, *y));
                },
                Wire::V(schematic::WireV { x, y1, y2}) => {
                    self.wire(Vector2::new(*x, *y1), Vector2::new(*x, *y2));
                },
            }
        }
        let last = wiring.last;
        let h = (last.x - self.cursor.x).abs();
        let v = (last.y - self.cursor.y).abs();
        match (wiring.segments.last(), h > v) {
            (Some(Wire::V(_)), _) | (None, true) => {
                let (x1, x2) = ord(last.x, self.cursor.x);
                self.wire(Vector2::new(x1, last.y), Vector2::new(x2, last.y));
                let (y1, y2) = ord(last.y, self.cursor.y);
                self.wire(Vector2::new(self.cursor.x, y1), Vector2::new(self.cursor.x, y2));
            },
            (Some(Wire::H(_)), _) | (None, false) => {
                let (y1, y2) = ord(last.y, self.cursor.y);
                self.wire(Vector2::new(last.x, y1), Vector2::new(last.x, y2));
                let (x1, x2) = ord(last.x, self.cursor.x);
                self.wire(Vector2::new(x1, self.cursor.y), Vector2::new(x2, self.cursor.y));
            },
        }
    }

    pub fn draw(&mut self) {
        self.draw_schematic();
        let state = std::mem::replace(&mut self.tool_state, ToolState::Selection);
        match &state {
            ToolState::Wiring(wiring) => {
                self.draw_wiring(&wiring);
                self.draw_cursor();
            },
            ToolState::ReadyToWire => {
                self.draw_cursor();
            },
            ToolState::Selection => {},
        }
        self.tool_state = state;
        self.backend.draw(&self.draw_list).unwrap();
    }

    pub fn set_frame_size(&mut self, w: u32, h: u32) {
        self.draw_list.frame_size = Vector2::new(w, h);
    }
}
