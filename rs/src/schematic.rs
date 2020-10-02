use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Schematic {
    junctions: Vec<Junction>,
    wires_h: Vec<WireH>,
    wires_v: Vec<WireV>,
}

#[derive(Serialize, Deserialize)]
pub struct Junction {
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
pub struct WireH {
    pub y: i32,
    pub x1: i32,
    pub x2: i32,
}

#[derive(Serialize, Deserialize)]
pub struct WireV {
    pub x: i32,
    pub y1: i32,
    pub y2: i32,
}
