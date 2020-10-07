mod draw_list;
mod io;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;
use super::schematic;
use super::backend::GolemBackend;
pub use draw_list::{Color, DrawList, DrawCmd};
pub use io::Io;

#[wasm_bindgen]
pub struct Cad {
    backend: GolemBackend,
    transform: Transform,
    grid_size: f32,
    draw_list: DrawList,
    cursor: Vector2<i32>,
    tool_state: ToolState,
    sch_state: schematic::State,
}

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

struct Transform {
    scale: f32,
    translate: Vector2<f32>,
    screen_size: Vector2<u32>,
}

impl Transform {
    fn pan_zoom(&mut self, pan: Vector2<f32>, origin: Vector2<f32>, zoom: f32) {
        self.translate = self.translate.scale(zoom) - origin.scale(zoom) + origin + pan;
        self.scale *= zoom;
    }

    fn screen_to_world(&self, screen: Vector2<f32>) -> Vector2<f32> {
        (screen - self.translate).unscale(self.scale)
    }

    fn world_to_screen(&self, world: Vector2<f32>) -> Vector2<f32> {
        world.scale(self.scale) + self.translate
    }

    fn viewbox(&self) -> (Vector2<f32>, Vector2<f32>) {
        let top_left = self.screen_to_world(Vector2::zeros());
        let frame_size: Vector2<f32> = nalgebra::convert(self.screen_size);
        let bottom_right = self.screen_to_world(frame_size);
        (top_left, bottom_right)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: 1.,
            translate: Vector2::zeros(),
            screen_size: Vector2::new(1, 1),
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
            transform: Transform::default(),
            grid_size: 50.,
            draw_list,
            cursor: Vector2::zeros(),
            tool_state: ToolState::Selection,
            sch_state: schematic::State::default(),
        }
    }

    fn process_pan_zoom(&mut self, io: &Io) {
        let pan = -io.wheel;
        let origin = io.mouse;
        let mut zoom = 1. - io.wheel_pinch * 0.02;
        if self.transform.scale * zoom < 0.1 {
            zoom = 0.1 / self.transform.scale;
        } else if self.transform.scale * zoom > 16.0 {
            zoom = 16.0 / self.transform.scale;
        }
        self.transform.pan_zoom(pan, origin, zoom);
    }

    fn process_cursor(&mut self, io: &Io) {
        let w = self.transform.screen_to_world(io.mouse);
        let g = self.world_to_grid(w);
        let rounded = g.map(|f| f.round() as i32);
        self.cursor = rounded;
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
        self.transform.screen_size = io.screen_size;
        self.process_pan_zoom(&io);
        self.process_cursor(&io);
        self.process_events(&io);
        io.reset();
        self.draw_list.clear();
        self.draw_list.scale = self.transform.scale;
        self.draw_list.translate = self.transform.translate;
        self.draw_list.screen_size = self.transform.screen_size;
    }

    fn draw_grid(&mut self) {
        let size = 2.0 / self.transform.scale;
        let base_gray = 0.7;
        let (top_left, bottom_right) = self.grid_viewbox();
        let mut step = 1;
        let mut ofs_x = 0;
        let mut ofs_y = 0;
        if self.transform.scale < 0.3 {
            step = 10;
            ofs_x = top_left.x % step;
            ofs_y = top_left.y % step;
        }
        for y in (top_left.y - ofs_y..bottom_right.y).step_by(step as usize) {
            let y_bold = if y % (10 * step) == 0 { 0.2 } else { 0.0 };
            for x in (top_left.x - ofs_x..bottom_right.x).step_by(step as usize) {
                let x_bold = if x % (10 * step) == 0 { 0.2 } else { 0.0 };
                let p = self.grid_to_world(Vector2::new(x, y));
                let rgb = base_gray - y_bold - x_bold;
                let col = Color::new(rgb, rgb, rgb, 1.);
                self.draw_list.add_square(p, size, col);
            }
        }
    }

    fn draw_cursor(&mut self) {
        let p = self.grid_to_world(self.cursor);
        let col = Color::new(0., 0., 0., 1.);
        let thickness = 1.0 / self.transform.scale;
        let half_len = 35. / self.transform.scale;
        self.draw_list.add_line(
            Vector2::new(p.x - half_len, p.y),
            Vector2::new(p.x + half_len, p.y),
            col,
            thickness,
        );
        self.draw_list.add_line(
            Vector2::new(p.x, p.y - half_len),
            Vector2::new(p.x, p.y + half_len),
            col,
            thickness,
        );
    }

    fn draw_schematic(&mut self) {
        let sch_state = std::mem::take(&mut self.sch_state);
        let (top_left, right_bottom) = self.grid_viewbox();
        let aabb = rstar::AABB::from_corners(top_left.into(), right_bottom.into());
        for wire in sch_state.wires_iter(&aabb) {
            self.wire(wire.from.into(), wire.to.into());
        }
        for junction in sch_state.junctions_iter(&aabb) {
            self.junction(junction.into());
        }
        self.sch_state = sch_state;
    }

    fn world_to_grid(&self, p: Vector2<f32>) -> Vector2<f32> {
        p.unscale(self.grid_size)
    }

    fn grid_to_world(&self, p: Vector2<i32>) -> Vector2<f32> {
        let p: Vector2<f32> = nalgebra::convert(p);
        p.scale(self.grid_size)
    }

    fn grid_viewbox(&self) -> (Vector2<i32>, Vector2<i32>) {
        let (a, b) = self.transform.viewbox();
        let a = a.unscale(self.grid_size).map(|n| n as i32);
        let b = b.unscale(self.grid_size).map(|n| n as i32);
        (a, b)
    }

    fn wire(&mut self, p1: Vector2<i32>, p2: Vector2<i32>) {
        let p1 = self.grid_to_world(p1);
        let p2 = self.grid_to_world(p2);
        let col = Color::new(0., 132. / 255., 0., 1.);
        self.draw_list.add_line(p1, p2, col, 6.);
    }

    fn junction(&mut self, p: Vector2<i32>) {
        let p = self.grid_to_world(p);
        let col = Color::new(0., 132. / 255., 0., 1.);
        self.draw_list.add_line(p, p, col, 40.);
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
        self.draw_grid();
        self.draw_schematic();
        let state = std::mem::replace(&mut self.tool_state, ToolState::Selection);
        match &state {
            ToolState::Wiring(wiring) => {
                self.draw_wiring(&wiring);
            },
            ToolState::ReadyToWire => {},
            ToolState::Selection => {},
        }
        self.draw_cursor();
        self.tool_state = state;
        self.backend.draw(&self.draw_list).unwrap();
    }
}
