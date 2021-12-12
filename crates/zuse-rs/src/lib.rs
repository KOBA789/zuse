mod backend;
mod schematic;
mod cad;
mod symbol;
mod font;
pub use cad::{Cad, Io};
pub use backend::GlowBackend;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
