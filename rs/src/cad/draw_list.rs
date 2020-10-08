use nalgebra::base::{Vector2, Vector4};

pub type Color = Vector4<f32>;

#[derive(Debug, Clone)]
pub struct DrawList {
    pub screen_size: Vector2<u32>,
    pub translate: Vector2<f32>,
    pub scale: f32,
    pub bg_color: Color,
    pub cmds: Vec<DrawCmd>,
    idx_buffer: Vec<u32>,
    vtx_buffer: Vec<Vert>,
}

impl DrawList {
    pub fn new(screen_size: Vector2<u32>) -> Self {
        Self {
            screen_size,
            translate: Vector2::zeros(),
            scale: 1.,
            bg_color: Color::new(1., 1., 1., 1.),
            cmds: vec![DrawCmd::default()],
            idx_buffer: vec![],
            vtx_buffer: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.cmds.clear();
        self.cmds.push(DrawCmd::default());
        self.idx_buffer.clear();
        self.vtx_buffer.clear();
    }

    pub fn new_layer(&mut self) {
        self.cmds.push(DrawCmd {
            idx_offset: self.idx_buffer.len(),
            vtx_offset: self.vtx_buffer.len(),
            num_of_elems: 0,
        });
    }

    fn reserve(&mut self, idx_count: usize, vtx_count: usize) {
        self.idx_buffer.reserve(idx_count);
        self.vtx_buffer.reserve(vtx_count);
    }

    fn push_vert(&mut self, vert: Vert) -> u32 {
        let idx = self.vtx_buffer.len() as u32;
        self.vtx_buffer.push(vert);
        idx
    }

    fn push_elem(&mut self, a: u32, b: u32, c: u32) {
        self.idx_buffer.push(a);
        self.idx_buffer.push(b);
        self.idx_buffer.push(c);
        self.cmds.last_mut().unwrap().num_of_elems += 1;
    }

    pub fn add_line_with_params(
        &mut self,
        p1: Vector2<f32>,
        p2: Vector2<f32>,
        col: Color,
        params: &LineParams,
    ) {
        self.reserve(params.idx_count(), params.vtx_count());

        let mut d = p2 - p1;
        d.try_normalize_mut(0.);
        d.scale_mut(params.half_thickness);

        let v0 = self.push_vert(Vert {
            pos: Vector2::new(p1.x + d.y, p1.y - d.x),
            col,
        });
        let v1 = self.push_vert(Vert {
            pos: Vector2::new(p2.x + d.y, p2.y - d.x),
            col,
        });
        let v2 = self.push_vert(Vert {
            pos: Vector2::new(p2.x - d.y, p2.y + d.x),
            col,
        });
        let v3 = self.push_vert(Vert {
            pos: Vector2::new(p1.x - d.y, p1.y + d.x),
            col,
        });
        self.push_elem(v0, v1, v2);
        self.push_elem(v0, v2, v3);

        if !params.cap_segments.is_empty() {
            let mut v_t = v1;
            let mut v_b = v3;
            let horizon = Vector2::new(-d.y, d.x);
            for r in params.cap_segments.iter() {
                if r.perp(&horizon) < 0. {
                    let v = self.push_vert(Vert { pos: p1 + r, col });
                    self.push_elem(v0, v_b, v);
                    v_b = v;
                    v_t = v1;
                } else {
                    let v = self.push_vert(Vert { pos: p2 + r, col });
                    self.push_elem(v_t, v2, v);
                    v_t = v;
                    v_b = v3;
                }
            }
        }
    }

    pub fn add_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>, col: Color, thickness: f32) {
        let resolution = thickness * self.scale;
        let cap_segment_count = if resolution <= 1.0 {
            0
        } else {
            (resolution * 1.5).ceil() as usize
        };
        let vtx_count = 4 + cap_segment_count;
        let idx_count = (2 + cap_segment_count) * 3;
        self.reserve(idx_count, vtx_count);

        let half_thickness = thickness * 0.5;
        let mut d = p2 - p1;
        d.try_normalize_mut(0.);
        d.scale_mut(half_thickness);

        let v0 = self.push_vert(Vert {
            pos: Vector2::new(p1.x + d.y, p1.y - d.x),
            col,
        });
        let v1 = self.push_vert(Vert {
            pos: Vector2::new(p2.x + d.y, p2.y - d.x),
            col,
        });
        let v2 = self.push_vert(Vert {
            pos: Vector2::new(p2.x - d.y, p2.y + d.x),
            col,
        });
        let v3 = self.push_vert(Vert {
            pos: Vector2::new(p1.x - d.y, p1.y + d.x),
            col,
        });
        self.push_elem(v0, v1, v2);
        self.push_elem(v0, v2, v3);

        if cap_segment_count > 0 {
            let mut v_t = v1;
            let mut v_b = v3;
            let horizon = Vector2::new(-d.y, d.x);
            for i in 0..=cap_segment_count {
                let rad = i as f32 * 2.0 / cap_segment_count as f32 * std::f32::consts::PI;
                let r = Vector2::new(rad.cos(), rad.sin()).scale(half_thickness);
                if r.perp(&horizon) < 0. {
                    let v = self.push_vert(Vert { pos: p1 + r, col });
                    self.push_elem(v0, v_b, v);
                    v_b = v;
                    v_t = v1;
                } else {
                    let v = self.push_vert(Vert { pos: p2 + r, col });
                    self.push_elem(v_t, v2, v);
                    v_t = v;
                    v_b = v3;
                }
            }
        }
    }

    pub fn add_circle(&mut self, p: Vector2<f32>, r: f32, col: Color, thickness: f32) {
        let resolution = thickness * self.scale;
        let half_thickness = thickness * 0.5;
        let segment_count = (r + resolution).ceil() as usize;
        let vtx_count = 2 * segment_count;
        let idx_count = (2 * segment_count) * 3;
        self.reserve(idx_count, vtx_count);

        let r_o = r + half_thickness;
        let r_i = r - half_thickness;
        let mut v_o0 = self.push_vert(Vert {
            pos: p + Vector2::new(r_o, 0.),
            col,
        });
        let mut v_i0 = self.push_vert(Vert {
            pos: p + Vector2::new(r_i, 0.),
            col,
        });
        for i in 1..=segment_count {
            let rad = i as f32 * 2.0 / segment_count as f32 * std::f32::consts::PI;
            let v = Vector2::new(rad.cos(), rad.sin());
            let v_o1 = self.push_vert(Vert {
                pos: p + v.scale(r_o),
                col,
            });
            let v_i1 = self.push_vert(Vert {
                pos: p + v.scale(r_i),
                col,
            });
            self.push_elem(v_o0, v_i0, v_o1);
            self.push_elem(v_o1, v_i1, v_i0);
            v_o0 = v_o1;
            v_i0 = v_i1;
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn add_square(&mut self, p: Vector2<f32>, size: f32, col: Color) {
        let half_size = size * 0.5;
        self.reserve(6, 4);
        let a = self.push_vert(Vert {
            pos: p + Vector2::new(-half_size, -half_size),
            col,
        });
        let b = self.push_vert(Vert {
            pos: p + Vector2::new(half_size, -half_size),
            col,
        });
        let c = self.push_vert(Vert {
            pos: p + Vector2::new(-half_size, half_size),
            col,
        });
        let d = self.push_vert(Vert {
            pos: p + Vector2::new(half_size, half_size),
            col,
        });
        self.push_elem(a, b, c);
        self.push_elem(b, c, d);
    }
}

impl DrawList {
    pub fn vertices(&self) -> &[f32] {
        unsafe {
            let len =
                std::mem::size_of::<Vert>() / std::mem::size_of::<f32>() * self.vtx_buffer.len();
            #[allow(clippy::transmute_ptr_to_ptr)]
            let ptr = std::mem::transmute::<*const Vert, *const f32>(self.vtx_buffer.as_ptr());
            &*std::ptr::slice_from_raw_parts(ptr, len)
        }
    }

    pub fn indices(&self) -> &[u32] {
        &self.idx_buffer
    }
}

pub struct LineParams {
    half_thickness: f32,
    cap_segments: Vec<Vector2<f32>>,
}

impl LineParams {
    pub fn new(scale: f32, thickness: f32) -> Self {
        let resolution = thickness * scale;
        let cap_segment_count = if resolution <= 1.0 {
            0
        } else {
            (resolution * 1.5).ceil() as usize
        };
        let half_thickness = thickness * 0.5;
        let cap_segments = (0..=cap_segment_count)
            .map(|i| {
                let rad = i as f32 * 2.0 / cap_segment_count as f32 * std::f32::consts::PI;
                Vector2::new(rad.cos(), rad.sin()).scale(half_thickness)
            })
            .collect();
        Self {
            half_thickness,
            cap_segments,
        }
    }

    fn vtx_count(&self) -> usize {
        4 + self.cap_segments.len()
    }

    fn idx_count(&self) -> usize {
        (2 + self.cap_segments.len()) * 3
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vert {
    pos: Vector2<f32>,
    //uv: Vector2<f32>,
    col: Color,
}

#[derive(Debug, Clone)]
pub struct DrawCmd {
    //clip_rect: Vector4<f32>,
    pub vtx_offset: usize,
    pub idx_offset: usize,
    pub num_of_elems: usize,
}

impl Default for DrawCmd {
    fn default() -> Self {
        Self {
            vtx_offset: 0,
            idx_offset: 0,
            num_of_elems: 0,
        }
    }
}
