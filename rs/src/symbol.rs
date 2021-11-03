use nalgebra::Vector2;
use rstar::AABB;
use serde::{Serialize, Deserialize};

use super::schematic::RotMirror;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Power,
    Contact,
    Coil,
}

impl Kind {
    pub fn aabb(self) -> AABB<[i32; 2]> {
        match self {
            Kind::Power => *power::AABB,
            Kind::Contact => *contact::AABB,
            Kind::Coil => *coil::AABB
        }
    }

    pub fn pads(self) -> &'static Pads {
        match self {
            Kind::Power => &power::PADS,
            Kind::Contact => &contact::PADS,
            Kind::Coil => &coil::PADS,
        }
    }

    pub fn can_rotate(self) -> bool {
        match self {
            Kind::Power => false,
            Kind::Contact => true,
            Kind::Coil => false,
        }
    }

    pub fn can_mirror(self) -> bool {
        match self {
            Kind::Power => false,
            Kind::Contact => true,
            Kind::Coil => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Draw {
    Line(Vector2<f32>, Vector2<f32>, f32),
    Circle(Vector2<f32>, f32, f32),
}

impl Draw {
    pub fn transform(&self, rot_mirror: RotMirror, translate: Vector2<i32>) -> Draw {
        let t: Vector2<f32> = nalgebra::convert(translate);
        match self {
            Draw::Line(p1, p2, thickness) => Draw::Line(
                rot_mirror.apply_f32(*p1) + t,
                rot_mirror.apply_f32(*p2) + t,
                *thickness,
            ),
            Draw::Circle(p, r, thickness) => {
                Draw::Circle(rot_mirror.apply_f32(*p) + t, *r, *thickness)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pad {
    pub position: Vector2<i32>,
    pub name: &'static str,
}

impl Pad {
    pub fn transform(&self, rot_mirror: RotMirror, translate: Vector2<i32>) -> Pad {
        Pad {
            position: rot_mirror.apply(self.position) + translate,
            name: self.name
        }
    }
}

#[derive(Debug)]
pub struct Pads(Vec<Pad>);
impl Pads {
    pub fn new(vec: Vec<Pad>) -> Self {
        Pads(vec)
    }

    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&Pad> {
        self.0.iter().find(|pad| pad.name == name)
    }

    pub fn transform(&self, rot_mirror: RotMirror, translate: Vector2<i32>) -> impl Iterator<Item = Pad> + '_ {
        self.iter().map(move |pad| pad.transform(rot_mirror, translate))
    }
}
impl std::ops::Deref for Pads {
    type Target = [Pad];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub mod power {
    use super::{Draw, Pad, Pads};
    use nalgebra::Vector2;
    lazy_static::lazy_static! {
        pub static ref AABB: rstar::AABB<[i32; 2]> = rstar::AABB::from_corners([-50, -100], [50, 0]);
        pub static ref PADS: Pads = Pads::new(vec![
            Pad {
                name: "V+",
                position: Vector2::zeros(),
            },
        ]);
        pub static ref DRAW: Vec<Draw> = vec![
            Draw::Line([0., -100.].into(), [0., 0.].into(), 6.),
            Draw::Line([0., -100.].into(), [-1. / (3.0f32).sqrt() * 50., -50.].into(), 6.),
            Draw::Line([0., -100.].into(), [1. / (3.0f32).sqrt() * 50., -50.].into(), 6.),
        ];
    }
}

pub mod contact {
    use super::{Draw, Pad, Pads};
    use nalgebra::Vector2;
    lazy_static::lazy_static! {
        pub static ref AABB: rstar::AABB<[i32; 2]> = rstar::AABB::from_corners([-50, -100], [50, 100]);
        pub static ref PADS: Pads = Pads::new(vec![
            Pad {
                name: "C",
                position: [0, -100].into(),
            },
            Pad {
                name: "A",
                position: [-50, 100].into(),
            },
            Pad {
                name: "B",
                position: [50, 100].into(),
            },
        ]);
        static ref STATIC_DRAW: Vec<Draw> = vec![
            Draw::Line([0., -100.].into(), [0., -60.].into(), 6.),
            Draw::Line([-50., 60.].into(), [-50., 100.].into(), 6.),
            Draw::Line([50., 60.].into(), [50., 100.].into(), 6.),
            Draw::Circle([0., -50.].into(), 10., 6.),
            Draw::Circle([-50., 50.].into(), 10., 6.),
            Draw::Line([50., 50.].into(), [50., 50.].into(), 26.),
        ];
        static ref MOVING_CONTACT_LINE: (Vector2<f32>, Vector2<f32>) = {
            let a = -50f32; // B contact x - C contact x
            let b = -100f32; // B contact y - C contact y
            let r = 16f32; // B contact outer radius + half-thickness of moving contact line
            let x = r * (a * r + b * (a * a + b * b - r * r).sqrt()) / (a * a + b * b) + 50.;
            let y = r * (b * r - a * (a * a + b * b - r * r).sqrt()) / (a * a + b * b) + 50.;
            let s = (Vector2::new(x, y) - Vector2::new(0., -50.)).normalize().scale(10.) + Vector2::new(0., -50.);
            (s, [x, y].into())
        };
    }

    pub fn draw(a: bool, b: bool) -> impl Iterator<Item = Draw> {
        STATIC_DRAW
            .iter()
            .cloned()
            .chain(std::iter::once_with(move || {
                let mut p1 = MOVING_CONTACT_LINE.0;
                let mut p2 = MOVING_CONTACT_LINE.1;
                #[allow(clippy::branches_sharing_code)]
                if a == b {
                    p1.x = 0.;
                    p2.x = 0.;
                    Draw::Line(p1, p2, 6.)
                } else if a {
                    p1.x = -p1.x;
                    p2.x = -p2.x;
                    Draw::Line(p1, p2, 6.)
                } else {
                    Draw::Line(p1, p2, 6.)
                }
            }))
    }
}

pub mod coil {
    use super::{Draw, Pad, Pads};
    lazy_static::lazy_static! {
        pub static ref AABB: rstar::AABB<[i32; 2]> = rstar::AABB::from_corners([-50, -100], [50, 150]);
        pub static ref PADS: Pads = Pads::new(vec![
            Pad {
                name: "N",
                position: [0, -100].into(),
            },
        ]);
        static ref STATIC_DRAW: Vec<Draw> = vec![
            Draw::Line([0., -100.].into(), [0., -50.].into(), 6.),
            Draw::Line([0., 50.].into(), [0., 100.].into(), 6.),
            Draw::Circle([0., 0.].into(), 50., 6.),
            Draw::Line([-25., 100.].into(), [25., 100.].into(), 6.),
            Draw::Line([-25., 100.].into(), [0., 1. / (2.0f32).sqrt() * 50. + 100.].into(), 6.),
            Draw::Line([25., 100.].into(), [0., 1. / (2.0f32).sqrt() * 50. + 100.].into(), 6.),
        ];
    }

    pub fn draw(state: bool) -> impl Iterator<Item = Draw> {
        STATIC_DRAW
            .iter()
            .cloned()
            .chain(std::iter::once_with(move || {
                if state {
                    Draw::Circle([0., 0.].into(), 25., 6.)
                } else {
                    Draw::Circle([0., 0.].into(), 50., 6.)
                }
            }))
    }
}
