mod draw_list;
mod io;

use crate::symbol;

use super::backend::GlowBackend;
use super::font::FONT;
use super::schematic;
pub use draw_list::{Color, DrawCmd, DrawList};
pub use io::Io;
use nalgebra::Vector2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Cad {
    backend: GlowBackend,
    transform: Transform,
    grid_size: u32,
    draw_list: DrawList,
    cursor: Vector2<i32>,
    pointer: Vector2<i32>,
    tool_state: ToolState,
    sch_state: schematic::State,
    circuit: Option<zuse_core::Circuit>,
}

enum ToolState {
    Selection,
    ReadyToWire,
    Wiring(Wiring),
    PlacingComponent(symbol::Kind, schematic::RotMirror),
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
            }
            (Some(Wire::H(_)), _) | (None, false) => {
                let (y1, y2) = ord(self.last.y, cursor.y);
                let wire = Wire::V(schematic::WireV {
                    x: self.last.x,
                    y1,
                    y2,
                });
                self.last = Vector2::new(self.last.x, cursor.y);
                self.segments.push(wire);
            }
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

    #[allow(dead_code)]
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
    pub fn new(backend: GlowBackend) -> Self {
        let draw_list = DrawList::new(Vector2::new(0, 0));
        Self {
            backend,
            transform: Transform::default(),
            grid_size: 50,
            draw_list,
            cursor: Vector2::zeros(),
            pointer: Vector2::zeros(),
            tool_state: ToolState::Selection,
            sch_state: schematic::State::default(),
            circuit: None,
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
        self.pointer = w.map(|f| f.round() as i32);
        let snapped = w
            .unscale(self.grid_size as f32)
            .map(|f| f.round() as i32 * self.grid_size as i32);
        self.cursor = snapped;
    }

    fn process_event_tool_selection(&mut self, event: &io::Event) -> (bool, Option<ToolState>) {
        match event {
            io::Event::Keydown(key) => match key.as_str() {
                "w" => (false, Some(ToolState::Wiring(Wiring::start(self.cursor)))),
                "p" => (
                    false,
                    Some(ToolState::PlacingComponent(
                        symbol::Kind::Power,
                        Default::default(),
                    )),
                ),
                "s" => (
                    false,
                    Some(ToolState::PlacingComponent(
                        symbol::Kind::Contact,
                        Default::default(),
                    )),
                ),
                "c" => (
                    false,
                    Some(ToolState::PlacingComponent(
                        symbol::Kind::Coil,
                        Default::default(),
                    )),
                ),
                "d" => {
                    self.sch_state
                        .delete_at_point(self.pointer.into(), self.grid_size as i32 / 4);
                    (false, None)
                }
                "r" => {
                    self.sch_state
                        .rotate_component_at_point(self.pointer, self.grid_size as i32 / 4);
                    (false, None)
                }
                "y" => {
                    self.sch_state
                        .mirror_component_at_point(self.pointer, self.grid_size as i32 / 4);
                    (false, None)
                }
                _ => (true, None),
            },
            io::Event::DoubleClick(0) => {
                if self.circuit.is_some() {
                    return (false, None);
                }
                if let Some(comp) = self.sch_state.components_iter_mut(rstar::AABB::from_point(self.cursor.into())).next() {
                    if let Some(new_label) = web_sys::window().unwrap().prompt_with_message_and_default("Label", &comp.label).unwrap() {
                        comp.label = new_label;
                    }
                }
                (false, None)
            },
            io::Event::Click(0) => {
                if let Some(circuit) = &mut self.circuit {
                    if let Some(component) = self.sch_state.components_iter(rstar::AABB::from_point(self.cursor.into())).next() {
                        if component.symbol == symbol::Kind::Contact {
                            let state = circuit.get_state(&format!("{}.A", &component.label)).unwrap_or(false);
                            let a = !state;
                            let b = !a;
                            circuit.set_state(&format!("{}.A", &component.label), a);
                            circuit.set_state(&format!("{}.B", &component.label), b);
                        }
                    }
                }
                (false, None)
            },
            _ => (true, None),
        }
    }

    fn process_event_tool_ready_to_wire(&mut self, event: &io::Event) -> (bool, Option<ToolState>) {
        match event {
            io::Event::Click(0) => (false, Some(ToolState::Wiring(Wiring::start(self.cursor)))),
            _ => (true, None),
        }
    }

    fn process_event_tool_wiring(
        &mut self,
        event: &io::Event,
        wiring: &mut Wiring,
    ) -> (bool, Option<ToolState>) {
        match event {
            io::Event::Keydown(key) if key == "w" => {
                wiring.add_segment(self.cursor);
                (false, None)
            }
            io::Event::Click(0) => {
                wiring.add_segment(self.cursor);
                (false, None)
            }
            io::Event::DoubleClick(0) => {
                for wire in &wiring.segments {
                    match wire {
                        Wire::H(wire_h) => {
                            self.sch_state.add_wire(wire_h.clone());
                        }
                        Wire::V(wire_v) => {
                            self.sch_state.add_wire(wire_v.clone());
                        }
                    }
                }
                (false, Some(ToolState::ReadyToWire))
            }
            _ => (true, None),
        }
    }

    fn process_event_tool_placing_component(
        &mut self,
        event: &io::Event,
        symbol: &mut symbol::Kind,
        rot_mirror: &mut schematic::RotMirror,
    ) -> (bool, Option<ToolState>) {
        match event {
            io::Event::Click(0) => {
                match symbol {
                    symbol::Kind::Power => {
                        let power = schematic::Component::new(
                            self.cursor,
                            symbol::Kind::Power,
                            *rot_mirror,
                            "V+".to_string(),
                        );
                        self.sch_state.add_component(power);
                    }
                    symbol::Kind::Contact => {
                        let contact = schematic::Component::new(
                            self.cursor,
                            symbol::Kind::Contact,
                            *rot_mirror,
                            "R".to_string(),
                        );
                        self.sch_state.add_component(contact);
                    }
                    symbol::Kind::Coil => {
                        let coil = schematic::Component::new(
                            self.cursor,
                            symbol::Kind::Coil,
                            *rot_mirror,
                            "R".to_string(),
                        );
                        self.sch_state.add_component(coil);
                    }
                }
                (false, Some(ToolState::Selection))
            }
            io::Event::Keydown(key) if key == "r" => {
                if symbol.can_rotate() {
                    *rot_mirror = rot_mirror.rotate_r();
                }
                (false, None)
            }
            io::Event::Keydown(key) if key == "y" => {
                if symbol.can_mirror() {
                    *rot_mirror = rot_mirror.mirror();
                }
                (false, None)
            }
            _ => (true, None),
        }
    }

    fn process_event_tool(&mut self, event: &io::Event) -> bool {
        let mut tool_state = std::mem::replace(&mut self.tool_state, ToolState::Selection);
        let (prevent_default, next_state) = match &mut tool_state {
            ToolState::Selection => self.process_event_tool_selection(event),
            ToolState::ReadyToWire => self.process_event_tool_ready_to_wire(event),
            ToolState::Wiring(wiring) => self.process_event_tool_wiring(event, wiring),
            ToolState::PlacingComponent(symbol, rot_mirror) => {
                self.process_event_tool_placing_component(event, symbol, rot_mirror)
            }
        };
        if let Some(next_state) = next_state {
            self.tool_state = next_state;
        } else {
            self.tool_state = tool_state;
        }
        prevent_default
    }

    fn process_event(&mut self, event: &io::Event) {
        if !self.process_event_tool(event) {
            return;
        }
        match event {
            io::Event::Keydown(key) if key == "Escape" => {
                self.tool_state = ToolState::Selection;
            }
            _ => {}
        }
    }

    fn process_events(&mut self, io: &Io) {
        for event in &io.events {
            self.process_event(event);
        }
    }

    pub fn new_frame(&mut self, io: &mut Io) {
        self.transform.screen_size = io.screen_size;
        let pixel_ratio = io.pixel_ratio;
        self.process_pan_zoom(io);
        self.process_cursor(io);
        self.process_events(io);
        io.reset();
        self.draw_list.clear();
        self.draw_list.pixel_ratio = pixel_ratio;
        self.draw_list.scale = self.transform.scale;
        self.draw_list.translate = self.transform.translate;
        self.draw_list.screen_size = self.transform.screen_size;
    }

    fn draw_grid(&mut self) {
        let size = 2.0 / self.transform.scale;
        let base_gray = 0.7;
        let (top_left, bottom_right) = self.grid_viewbox();
        let mut step = self.grid_size as i32;
        if self.transform.scale < 0.3 {
            step *= 10;
        }
        let ofs_x = top_left.x % step;
        let ofs_y = top_left.y % step;
        for y in (top_left.y - ofs_y..bottom_right.y).step_by(step as usize) {
            let y_bold = if y % (10 * step) == 0 { 0.2 } else { 0.0 };
            for x in (top_left.x - ofs_x..bottom_right.x).step_by(step as usize) {
                let x_bold = if x % (10 * step) == 0 { 0.2 } else { 0.0 };
                let p = nalgebra::convert(Vector2::new(x, y));
                let rgb = base_gray - y_bold - x_bold;
                let col = Color::new(rgb, rgb, rgb, 1.);
                self.draw_list.add_square(p, size, col);
            }
        }
    }

    fn draw_cursor(&mut self) {
        let p: Vector2<f32> = nalgebra::convert(self.cursor);
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
        for component in sch_state.components_iter(aabb) {
            self.component(component);
            self.text((component.position + Vector2::new(50, 0)).map(|n| n as f32), &component.label);
        }
        for (p, rc) in sch_state.junctions_iter(&aabb) {
            self.junction(p, rc);
        }
        self.sch_state = sch_state;
    }

    fn grid_viewbox(&self) -> (Vector2<i32>, Vector2<i32>) {
        let (a, b) = self.transform.viewbox();
        let a = a.map(|n| n as i32);
        let b = b.map(|n| n as i32);
        (a, b)
    }

    fn wire(&mut self, p1: Vector2<i32>, p2: Vector2<i32>) {
        let p1 = nalgebra::convert(p1);
        let p2 = nalgebra::convert(p2);
        let col = Color::new(0., 132. / 255., 0., 1.);
        self.draw_list.add_line(p1, p2, col, 6.);
    }

    fn junction(&mut self, p: Vector2<i32>, rc: u8) {
        let p = nalgebra::convert(p);
        let col = Color::new(0., 132. / 255., 0., 1.);
        if rc >= 3 {
            self.draw_list.add_line(p, p, col, 40.);
        } else if rc == 1 {
            self.draw_list.add_circle(p, 10., col, 1.);
        }
    }

    fn component(&mut self, component: &schematic::Component) {
        let rot_mirror = component.rot_mirror;
        let position = component.position;
        let col = Color::new(0.51, 0., 0., 1.);
        match component.symbol {
            symbol::Kind::Power => {
                let draw_iter = symbol::power::DRAW
                    .iter()
                    .map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
            symbol::Kind::Contact => {
                let a = self.circuit.as_ref().and_then(|c| c.get_state(&format!("{}.A", &component.label))).unwrap_or(false);
                let b = self.circuit.as_ref().and_then(|c| c.get_state(&format!("{}.B", &component.label))).unwrap_or(true);
                let draw_iter =
                    symbol::contact::draw(a, b).map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
            symbol::Kind::Coil => {
                let a = self.circuit.as_ref().and_then(|c| c.get_state(&format!("{}.A", &component.label))).unwrap_or(false);
                let draw_iter =
                    symbol::coil::draw(a).map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
        }
    }

    fn draw_symbol(&mut self, col: Color, draw_iter: impl Iterator<Item = symbol::Draw>) {
        for draw in draw_iter {
            match draw {
                symbol::Draw::Line(p1, p2, thickness) => {
                    self.draw_list.add_line(p1, p2, col, thickness);
                }
                symbol::Draw::Circle(p, r, thickness) => {
                    self.draw_list.add_circle(p, r, col, thickness);
                }
            }
        }
    }

    fn draw_wiring(&mut self, wiring: &Wiring) {
        for segment in &wiring.segments {
            match segment {
                Wire::H(schematic::WireH { y, x1, x2 }) => {
                    self.wire(Vector2::new(*x1, *y), Vector2::new(*x2, *y));
                }
                Wire::V(schematic::WireV { x, y1, y2 }) => {
                    self.wire(Vector2::new(*x, *y1), Vector2::new(*x, *y2));
                }
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
                self.wire(
                    Vector2::new(self.cursor.x, y1),
                    Vector2::new(self.cursor.x, y2),
                );
            }
            (Some(Wire::H(_)), _) | (None, false) => {
                let (y1, y2) = ord(last.y, self.cursor.y);
                self.wire(Vector2::new(last.x, y1), Vector2::new(last.x, y2));
                let (x1, x2) = ord(last.x, self.cursor.x);
                self.wire(
                    Vector2::new(x1, self.cursor.y),
                    Vector2::new(x2, self.cursor.y),
                );
            }
        }
    }

    fn draw_placing_component(&mut self, symbol: symbol::Kind, rot_mirror: schematic::RotMirror) {
        let position = self.cursor;
        let col = Color::new(0.51, 0., 0., 0.5);
        match symbol {
            symbol::Kind::Power => {
                let draw_iter = symbol::power::DRAW
                    .iter()
                    .map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
            symbol::Kind::Contact => {
                let draw_iter =
                    symbol::contact::draw(false, true).map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
            symbol::Kind::Coil => {
                let state = false; // TODO: use simulator's state
                let draw_iter =
                    symbol::coil::draw(state).map(|draw| draw.transform(rot_mirror, position));
                self.draw_symbol(col, draw_iter);
            }
        }
    }

    fn text(&mut self, p: Vector2<f32>, text: &str) {
        let mut advance = Vector2::new(0.0f32, 0.0);
        for char in text.chars() {
            if let Some(glyph) = FONT.glyph(char) {
                for (p1, p2) in glyph {
                    let p1 = p + (advance + p1).scale(4.5454);
                    let p2 = p + (advance + p2).scale(4.5454);
                    let col = Color::new(0., 0., 0., 1.);
                    self.draw_list.add_line(p1, p2, col, 3.0);
                }
            }
            advance += Vector2::new(FONT.advance(), 0.0);
        }
    }

    pub fn draw(&mut self) {
        if let Some(circuit) = &mut self.circuit {
            circuit.simulate();
        }
        self.draw_grid();
        self.draw_schematic();
        let state = std::mem::replace(&mut self.tool_state, ToolState::Selection);
        match &state {
            ToolState::Wiring(wiring) => {
                self.draw_wiring(wiring);
                self.draw_cursor();
            }
            ToolState::ReadyToWire => {
                self.draw_cursor();
            }
            ToolState::Selection => {}
            ToolState::PlacingComponent(symbol, rot_mirror) => {
                self.draw_list.new_layer();
                self.draw_placing_component(*symbol, *rot_mirror);
                self.draw_cursor();
            }
        }
        self.tool_state = state;
        self.backend.draw(&self.draw_list).unwrap();
    }

    pub fn save_schematic(&self) -> String {
        serde_json::to_string(&self.sch_state).unwrap()
    }
    pub fn load_schematic(&mut self, json: String) {
        self.sch_state = serde_json::from_str(&json).unwrap();
    }
    pub fn start_simulation(&mut self) {
        let netlist = self.sch_state.build_netlist();
        let spec = zuse_core::compile(&netlist);
        let mut circuit = spec.build();
        circuit.simulate();
        self.circuit = Some(circuit);
    }
    pub fn stop_simulation(&mut self) {
        self.circuit = None;
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq)]
pub struct ComponentMetadata {
    position: (i32, i32),
    label: String,
    symbol: symbol::Kind,
}

#[wasm_bindgen]
impl ComponentMetadata {
    pub fn label(&self) -> String {
        self.label.clone()
    }
}
