use std::collections::HashMap;

use nalgebra::Vector2;

const FONT_JSON: &[u8] = include_bytes!("./font.json");

pub type Segment = (Vector2<f32>, Vector2<f32>);
pub type Glyph = Vec<Segment>;

pub struct Font {
    advance: f32,
    glyphs: HashMap<char, Glyph>,
}

impl Font {
    fn load(json: &[u8], advance: f32) -> Self {
        let glyphs: HashMap<char, Vec<((f32, f32), (f32, f32))>> =
            serde_json::from_slice(FONT_JSON).unwrap();
        let glyphs = glyphs
            .into_iter()
            .map(|(char, segments)| {
                (
                    char,
                    segments
                        .into_iter()
                        .map(|(p1, p2)| (Vector2::new(p1.0, p1.1), Vector2::new(p2.0, p2.1)))
                        .collect(),
                )
            })
            .collect();
        Self { advance, glyphs }
    }

    pub fn glyph(&self, char: char) -> Option<&Glyph> {
        self.glyphs.get(&char)
    }

    pub fn advance(&self) -> f32 {
        self.advance
    }
}

lazy_static::lazy_static! {
    pub static ref FONT: Font = Font::load(FONT_JSON, 11.0);
}
