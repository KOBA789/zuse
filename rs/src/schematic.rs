use std::collections::HashMap;

use itertools::Itertools;
use nalgebra::Vector2;
use rstar::{AABB, RTree, RTreeObject, primitives::{GeomWithData, Line}};
use serde::{Deserialize, Serialize};

use crate::symbol;

#[derive(Serialize, Deserialize)]
pub struct Schematic {
    wires_h: Vec<WireH>,
    wires_v: Vec<WireV>,
}

pub trait Rectangular {
    type Orthogonal: Rectangular;
    const PARA_AXIS: usize;
    const PERP_AXIS: usize;

    #[inline]
    fn is_para(line: Line<[i32; 2]>) -> bool {
        line.from[Self::PERP_AXIS] == line.to[Self::PERP_AXIS]
    }

    #[inline]
    fn start(line: Line<[i32; 2]>) -> i32 {
        line.from[Self::PARA_AXIS]
    }

    #[inline]
    fn end(line: Line<[i32; 2]>) -> i32 {
        line.to[Self::PARA_AXIS]
    }

    #[inline]
    fn perp(line: Line<[i32; 2]>) -> i32 {
        line.from[Self::PERP_AXIS]
    }

    #[inline]
    fn line(start: i32, end: i32, perp: i32) -> Line<[i32; 2]> {
        let mut from = [0; 2];
        from[Self::PARA_AXIS] = start;
        from[Self::PERP_AXIS] = perp;
        let mut to = [0; 2];
        to[Self::PARA_AXIS] = end;
        to[Self::PERP_AXIS] = perp;
        Line::new(from, to)
    }

    #[allow(clippy::type_complexity)]
    fn split_if_needed(line: Line<[i32; 2]>, m: i32) -> Option<(Line<[i32; 2]>, Line<[i32; 2]>)> {
        let start = Self::start(line);
        let end = Self::end(line);
        if start == m || end == m {
            // corner
            return None;
        }
        // tangent
        // need to split
        let perp = Self::perp(line);
        let wire1 = Self::line(start, m, perp);
        let wire2 = Self::line(m, end, perp);
        Some((wire1, wire2))
    }
}

#[derive(Debug)]
pub struct Horizontal;
impl Rectangular for Horizontal {
    type Orthogonal = Vertical;
    const PARA_AXIS: usize = 0;
    const PERP_AXIS: usize = 1;
}

#[derive(Debug)]
pub struct Vertical;
impl Rectangular for Vertical {
    type Orthogonal = Horizontal;
    const PARA_AXIS: usize = 1;
    const PERP_AXIS: usize = 0;
}

pub trait Wire {
    type Axis: Rectangular;

    fn start(&self) -> i32;
    fn end(&self) -> i32;
    fn perp(&self) -> i32;

    #[inline]
    fn point_start(&self) -> [i32; 2] {
        let mut p = [0; 2];
        p[Self::Axis::PARA_AXIS] = self.start();
        p[Self::Axis::PERP_AXIS] = self.perp();
        p
    }

    #[inline]
    fn point_end(&self) -> [i32; 2] {
        let mut p = [0; 2];
        p[Self::Axis::PARA_AXIS] = self.end();
        p[Self::Axis::PERP_AXIS] = self.perp();
        p
    }

    #[inline]
    fn len(&self) -> u32 {
        (self.end() - self.start()) as u32
    }

    fn aabb(&self) -> AABB<[i32; 2]> {
        AABB::from_corners(self.point_start(), self.point_end())
    }

    fn aabb_start(&self) -> AABB<[i32; 2]> {
        AABB::from_point(self.point_start())
    }

    fn aabb_end(&self) -> AABB<[i32; 2]> {
        AABB::from_point(self.point_end())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WireH {
    pub y: i32,
    pub x1: i32,
    pub x2: i32,
}

impl Wire for WireH {
    type Axis = Horizontal;

    fn start(&self) -> i32 {
        self.x1
    }

    fn end(&self) -> i32 {
        self.x2
    }

    fn perp(&self) -> i32 {
        self.y
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WireV {
    pub x: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Wire for WireV {
    type Axis = Vertical;

    fn start(&self) -> i32 {
        self.y1
    }

    fn end(&self) -> i32 {
        self.y2
    }

    fn perp(&self) -> i32 {
        self.x
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Junctions {
    rtree: RTree<GeomWithData<[i32; 2], u8>>,
}

impl Junctions {
    fn incr_by(&mut self, p: [i32; 2], by: u8) -> u8 {
        if let Some(j) = self.rtree.locate_at_point_mut(&p) {
            j.data += by;
            j.data
        } else {
            self.rtree.insert(GeomWithData::new(p, by));
            1
        }
    }

    fn decr_by(&mut self, p: [i32; 2], by: u8) -> u8 {
        if let Some(j) = self.rtree.locate_at_point_mut(&p) {
            if j.data <= by {
                self.rtree.remove_at_point(&p);
                0
            } else {
                j.data -= by;
                j.data
            }
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct RotMirror(i8, i8, i8, i8);
impl RotMirror {
    pub fn rotate_r(self) -> RotMirror {
        let RotMirror(a, b, c, d) = self;
        RotMirror(-c, -d, a, b)
    }

    #[allow(dead_code)]
    pub fn rotate_l(self) -> RotMirror {
        let RotMirror(a, b, c, d) = self;
        RotMirror(c, d, -a, -b)
    }

    pub fn mirror(self) -> RotMirror {
        let RotMirror(a, b, c, d) = self;
        RotMirror(-a, -b, c, d)
    }

    #[allow(clippy::many_single_char_names)]
    pub fn apply(self, p: Vector2<i32>) -> Vector2<i32> {
        let x = p.x;
        let y = p.y;
        let RotMirror(a, b, c, d) = self;
        [a as i32 * x + b as i32 * y, c as i32 * x + d as i32 * y].into()
    }

    #[allow(clippy::many_single_char_names)]
    pub fn apply_f32(self, p: Vector2<f32>) -> Vector2<f32> {
        let RotMirror(a, b, c, d) = self;
        Vector2::new(
            a as f32 * p.x + b as f32 * p.y,
            c as f32 * p.x + d as f32 * p.y,
        )
    }
}

impl Default for RotMirror {
    fn default() -> Self {
        RotMirror(1, 0, 0, 1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Component {
    pub position: Vector2<i32>,
    aabb: AABB<[i32; 2]>,
    pub symbol: symbol::Kind,
    pub rot_mirror: RotMirror,
    #[serde(default)]
    pub label: String,
}

impl Component {
    pub fn new(
        position: Vector2<i32>,
        symbol: symbol::Kind,
        rot_mirror: RotMirror,
        label: String,
    ) -> Self {
        let aabb = symbol.aabb();
        let p1 = rot_mirror.apply(aabb.upper().into()) + position;
        let p2 = rot_mirror.apply(aabb.lower().into()) + position;
        let aabb = AABB::from_corners(p1.into(), p2.into());
        Self {
            position,
            aabb,
            symbol,
            rot_mirror,
            label,
        }
    }

    fn rot_mirror(&self, rot_mirror: RotMirror) -> Self {
        Self::new(self.position, self.symbol, rot_mirror, self.label.clone())
    }

    fn pads(&self) -> impl Iterator<Item = symbol::Pad> {
        self.symbol.pads().transform(self.rot_mirror, self.position)
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.rot_mirror == other.rot_mirror
    }
}

impl rstar::RTreeObject for Component {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    wires: RTree<Line<[i32; 2]>>,
    junctions: Junctions,
    components: RTree<Component>,
}

impl State {
    pub fn add_wire<W: Wire>(&mut self, new_wire: W) {
        if new_wire.len() == 0 {
            return;
        }

        let wires = self
            .wires
            .locate_in_envelope_intersecting(&new_wire.aabb())
            .filter(|&&wire| W::Axis::is_para(wire))
            .cloned()
            .collect::<Vec<_>>();
        let mut start = new_wire.start();
        let mut end = new_wire.end();
        for wire in wires {
            if start > W::Axis::start(wire) {
                start = W::Axis::start(wire);
            }
            if end < W::Axis::end(wire) {
                end = W::Axis::end(wire);
            }
            // remove all overwrapping wires
            // NOTE: this is inefficient. junctions and wires should be reused if possible
            //       however wires overwrap rarely enough practically
            self.junctions.decr_by(wire.from, 1);
            self.junctions.decr_by(wire.to, 1);
            self.wires.remove(&wire);
        }

        let start_side = self
            .wires
            .locate_in_envelope_intersecting(&new_wire.aabb_start());
        let end_side = self
            .wires
            .locate_in_envelope_intersecting(&new_wire.aabb_end());
        let both_side_ortho_wires = start_side
            .chain(end_side)
            .filter(|&&wire| <W::Axis as Rectangular>::Orthogonal::is_para(wire))
            .cloned()
            .collect::<Vec<_>>();
        for ortho_wire in both_side_ortho_wires {
            if let Some((wire1, wire2)) =
                <W::Axis as Rectangular>::Orthogonal::split_if_needed(ortho_wire, new_wire.perp())
            {
                self.wires.remove(&ortho_wire);
                self.wires.insert(wire1);
                self.wires.insert(wire2);
                self.junctions.incr_by(wire1.to, 2);
            }
        }

        let junctions = self
            .junctions
            .rtree
            .locate_in_envelope_intersecting(&new_wire.aabb())
            .map(|p| p.geom()[W::Axis::PARA_AXIS])
            .sorted();
        let vertices = std::iter::once(start)
            .chain(junctions)
            .chain(std::iter::once(end))
            .dedup();
        for (start, end) in vertices.tuple_windows() {
            let wire = W::Axis::line(start, end, new_wire.perp());
            self.wires.insert(wire);
            self.junctions.incr_by(wire.from, 1);
            self.junctions.incr_by(wire.to, 1);
        }
    }

    fn normalize_wires(&mut self, dirty_junctions: &[[i32; 2]]) {
        for &junction in dirty_junctions {
            let (wires_h, wires_v): (Vec<_>, Vec<_>) = self
                .wires
                .locate_in_envelope_intersecting(&AABB::from_point(junction))
                .cloned()
                .partition(|&wire| Horizontal::is_para(wire));
            match (wires_h.len(), wires_v.len()) {
                (2, 0) => {
                    self.wires.remove(&wires_h[0]);
                    self.wires.remove(&wires_h[1]);
                    self.junctions.decr_by(junction, 2);
                    let start = Horizontal::start(wires_h[0]).min(Horizontal::start(wires_h[1]));
                    let end = Horizontal::end(wires_h[0]).max(Horizontal::end(wires_h[1]));
                    let perp = Horizontal::perp(wires_h[0]);
                    self.wires.insert(Horizontal::line(start, end, perp));
                }
                (0, 2) => {
                    self.wires.remove(&wires_v[0]);
                    self.wires.remove(&wires_v[1]);
                    self.junctions.decr_by(junction, 2);
                    let start = Vertical::start(wires_v[0]).min(Vertical::start(wires_v[1]));
                    let end = Vertical::end(wires_v[0]).max(Vertical::end(wires_v[1]));
                    let perp = Vertical::perp(wires_v[0]);
                    self.wires.insert(Vertical::line(start, end, perp));
                }
                _ => {}
            }
        }
    }

    pub fn delete_at_point(&mut self, p: [i32; 2], size: i32) {
        let aabb = AABB::from_corners([p[0] - size, p[1] - size], [p[0] + size, p[1] + size]);
        let wires_to_delete = self
            .wires
            .locate_in_envelope_intersecting(&aabb)
            .cloned()
            .collect::<Vec<_>>();
        let mut dirty_junctions = vec![];
        for wire in wires_to_delete {
            self.wires.remove(&wire);
            let rc = self.junctions.decr_by(wire.from, 1);
            if rc == 2 {
                dirty_junctions.push(wire.from);
            }
            let rc = self.junctions.decr_by(wire.to, 1);
            if rc == 2 {
                dirty_junctions.push(wire.to);
            }
        }
        let components_to_delete = self
            .components
            .locate_in_envelope_intersecting(&aabb)
            .cloned()
            .collect::<Vec<_>>();
        for component in components_to_delete {
            self.delete_component(&component, &mut dirty_junctions);
        }
        self.normalize_wires(&dirty_junctions);
    }

    fn delete_component(&mut self, component: &Component, dirty_junctions: &mut Vec<[i32; 2]>) {
        self.components.remove(component);
        for pad in component.pads() {
            let p = pad.position.into();
            let rc = self.junctions.decr_by(p, 1);
            if rc == 2 {
                dirty_junctions.push(p);
            }
        }
    }

    pub fn rotate_component_at_point(&mut self, p: Vector2<i32>, size: i32) {
        let aabb = AABB::from_corners([p[0] - size, p[1] - size], [p[0] + size, p[1] + size]);
        let components_to_be_rotated = self
            .components
            .locate_in_envelope_intersecting(&aabb)
            .filter(|c| c.symbol.can_rotate())
            .cloned()
            .collect::<Vec<_>>();
        let mut dirty_junctions = vec![];
        for component in &components_to_be_rotated {
            self.delete_component(component, &mut dirty_junctions);
        }
        self.normalize_wires(&dirty_junctions);
        for component in components_to_be_rotated {
            let rotated_component = component.rot_mirror(component.rot_mirror.rotate_r());
            self.add_component(rotated_component);
        }
    }

    pub fn mirror_component_at_point(&mut self, p: Vector2<i32>, size: i32) {
        let aabb = AABB::from_corners([p[0] - size, p[1] - size], [p[0] + size, p[1] + size]);
        let components_to_be_rotated = self
            .components
            .locate_in_envelope_intersecting(&aabb)
            .filter(|c| c.symbol.can_mirror())
            .cloned()
            .collect::<Vec<_>>();
        let mut dirty_junctions = vec![];
        for component in &components_to_be_rotated {
            self.delete_component(component, &mut dirty_junctions);
        }
        self.normalize_wires(&dirty_junctions);
        for component in components_to_be_rotated {
            let rotated_component = component.rot_mirror(component.rot_mirror.mirror());
            self.add_component(rotated_component);
        }
    }

    pub fn add_component(&mut self, component: Component) -> bool {
        if self.components.locate_in_envelope_intersecting(&component.envelope()).next().is_some() {
            return false;
        }
        for pad in component.pads() {
            let p = pad.position;
            let contacting_wires = self
                .wires
                .locate_in_envelope_intersecting(&AABB::from_point(p.into()))
                .cloned()
                .collect::<Vec<_>>();
            for contacting_wire in contacting_wires {
                let wire_pair = if Horizontal::is_para(contacting_wire) {
                    Horizontal::split_if_needed(contacting_wire, p[Horizontal::PARA_AXIS])
                } else {
                    Vertical::split_if_needed(contacting_wire, p[Vertical::PARA_AXIS])
                };
                if let Some((wire1, wire2)) = wire_pair {
                    self.wires.remove(&contacting_wire);
                    self.wires.insert(wire1);
                    self.wires.insert(wire2);
                    self.junctions.incr_by(wire1.to, 2);
                }
            }
            self.junctions.incr_by(p.into(), 1);
        }
        self.components.insert(component);
        true
    }

    pub fn wires_iter(&self, aabb: &AABB<[i32; 2]>) -> impl Iterator<Item = &Line<[i32; 2]>> {
        self.wires.locate_in_envelope_intersecting(aabb)
    }

    pub fn junctions_iter<'a>(
        &'a self,
        aabb: &AABB<[i32; 2]>,
    ) -> impl Iterator<Item = (Vector2<i32>, u8)> + 'a {
        self.junctions
            .rtree
            .locate_in_envelope_intersecting(aabb)
            .map(|p| (Vector2::from(*p.geom()), p.data))
    }

    pub fn components_iter(&self, aabb: AABB<[i32; 2]>) -> impl Iterator<Item = &Component> {
        self.components.locate_in_envelope_intersecting(&aabb)
    }

    pub fn components_iter_mut(&mut self, aabb: AABB<[i32; 2]>) -> impl Iterator<Item = &mut Component> {
        self.components.locate_in_envelope_intersecting_mut(&aabb)
    }

    pub fn build_netlist(&self) -> zuse_core::net::Netlist {
        let mut max_net = 0;
        let mut net_alias = HashMap::<u32, u32>::new();
        let min_net = |net_alias: &HashMap<u32, u32>, mut net: u32| loop {
            match net_alias.get(&net) {
                Some(&less) => net = less,
                None => return net,
            }
        };
        let mut net_map = HashMap::<[i32; 2], u32>::new();
        for component in self.components.iter() {
            if let symbol::Kind::Power = component.symbol {
                net_map.insert(component.position.into(), 0);
            }
        }
        for wire in self.wires.iter() {
            let from_entry = net_map.get(&wire.from);
            let to_entry = net_map.get(&wire.to);
            match (from_entry, to_entry) {
                (None, None) => {
                    max_net += 1;
                    let new_net = max_net;
                    net_map.insert(wire.from, new_net);
                    net_map.insert(wire.to, new_net);
                }
                (None, Some(&net)) => {
                    net_map.insert(wire.from, net);
                }
                (Some(&net), None) => {
                    net_map.insert(wire.to, net);
                }
                (Some(&net1), Some(&net2)) => {
                    if net1 != net2 {
                        let min_net = min_net(&net_alias, net1.min(net2));
                        net_alias.insert(net1.max(net2), min_net);
                    }
                }
            }
        }
        let mut uni_net_map = HashMap::new();
        for (p, net) in net_map.into_iter() {
            uni_net_map.insert(p, min_net(&net_alias, net));
        }
        let mut relays = vec![];
        let mut switches = vec![];
        for component in self.components.iter() {
            match component.symbol {
                symbol::Kind::Power => {}
                symbol::Kind::Contact => {
                    let mut pads = component.pads();
                    let pad_c = pads.next().unwrap();
                    let pad_a = pads.next().unwrap();
                    let pad_b = pads.next().unwrap();
                    let pad_c_p: [i32; 2] = pad_c.position.into();
                    let pad_a_p: [i32; 2] = pad_a.position.into();
                    let pad_b_p: [i32; 2] = pad_b.position.into();
                    let pad_c_net = uni_net_map.get(&pad_c_p).copied().unwrap_or_else(|| {
                        max_net += 1;
                        max_net
                    });
                    let pad_a_net = uni_net_map.get(&pad_a_p).copied().unwrap_or_else(|| {
                        max_net += 1;
                        max_net
                    });
                    let pad_b_net = uni_net_map.get(&pad_b_p).copied().unwrap_or_else(|| {
                        max_net += 1;
                        max_net
                    });
                    switches.push(zuse_core::net::Switch {
                        state: format!("{}.A", component.label),
                        l: format!("N{}", pad_c_net),
                        r: format!("N{}", pad_a_net),
                    });
                    switches.push(zuse_core::net::Switch {
                        state: format!("{}.B", component.label),
                        l: format!("N{}", pad_c_net),
                        r: format!("N{}", pad_b_net),
                    });
                }
                symbol::Kind::Coil => {
                    let pad = component.pads().next().unwrap();
                    let p: [i32; 2] = pad.position.into();
                    let net = uni_net_map.get(&p).copied().unwrap_or_else(|| {
                        max_net += 1;
                        max_net
                    });
                    relays.push(zuse_core::net::Relay {
                        coil: format!("N{}", net),
                        a: format!("{}.A", &component.label),
                        b: format!("{}.B", &component.label),
                    });
                }
            }
        }
        zuse_core::net::Netlist { relays, switches }
    }
}
