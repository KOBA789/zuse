use super::cad::DrawList;
use anyhow::Result;
use glow::{Buffer, Context, HasContext, UniformLocation};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GlowBackend {
    gl: Context,
    projection_location: UniformLocation,
    position_location: u32,
    color_location: u32,
    vbo: Buffer,
    ebo: Buffer,
}

fn glow_error(s: String) -> anyhow::Error {
    anyhow::anyhow!("Glow Error: {}", s)
}

#[wasm_bindgen]
impl GlowBackend {
    #[wasm_bindgen(constructor)]
    pub fn from_webgl(webgl_context: web_sys::WebGl2RenderingContext) -> Self {
        let gl = glow::Context::from_webgl2_context(webgl_context);
        Self::new(gl).unwrap()
    }
}

impl GlowBackend {
    pub fn new(gl: Context) -> Result<Self> {
        unsafe {
            let vao = gl.create_vertex_array().map_err(glow_error)?;
            gl.bind_vertex_array(Some(vao));
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            let program = gl.create_program().map_err(glow_error)?;
            let vertex_shader_source = include_str!("shader.vert");
            let fragment_shader_source = include_str!("shader.frag");
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).map_err(glow_error)?;
            gl.shader_source(vertex_shader, vertex_shader_source);
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                return Err(anyhow::anyhow!(
                    "Glow Error: {}",
                    gl.get_shader_info_log(vertex_shader)
                ));
            }
            gl.attach_shader(program, vertex_shader);
            let fragment_shader = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(glow_error)?;
            gl.shader_source(fragment_shader, fragment_shader_source);
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                return Err(anyhow::anyhow!(
                    "Glow Error: {}",
                    gl.get_shader_info_log(fragment_shader)
                ));
            }
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(anyhow::anyhow!(
                    "Glow Error: {}",
                    gl.get_program_info_log(program)
                ));
            }
            gl.use_program(Some(program));
            gl.detach_shader(program, vertex_shader);
            gl.delete_shader(vertex_shader);
            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(fragment_shader);
            let projection_location = gl
                .get_uniform_location(program, "projection")
                .ok_or_else(|| anyhow::anyhow!("No projection uniform variable"))?;
            let position_location = gl
                .get_attrib_location(program, "vert_position")
                .ok_or_else(|| anyhow::anyhow!("No vert_position attribute"))?;
            let color_location = gl
                .get_attrib_location(program, "vert_color")
                .ok_or_else(|| anyhow::anyhow!("No vert_color attribute"))?;
            let vbo = gl.create_buffer().map_err(glow_error)?;
            let ebo = gl.create_buffer().map_err(glow_error)?;
            Ok(Self {
                gl,
                projection_location,
                position_location,
                color_location,
                vbo,
                ebo,
            })
        }
    }

    pub fn draw(&mut self, draw_list: &DrawList) -> Result<()> {
        let w = draw_list.screen_size.x as f32;
        let h = draw_list.screen_size.y as f32;
        let scale = draw_list.scale;
        let translate = draw_list.translate;
        let sx = (2. / w) * scale;
        let sy = (2. / h) * scale;
        let npx = 2. * translate.x / w + 1. / w;
        let npy = -2. * translate.y / h + 1. / h;
        #[rustfmt::skip]
        let projection = [
            sx, 0., 0., 0.,
            0., -sy, 0., 0.,
            0., 0., -1., 0.,
            npx - 1., npy + 1., 0., 1.,
        ];
        let vertices = draw_list.vertices();
        let indices = draw_list.indices();
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STREAM_DRAW,
            );
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STREAM_DRAW,
            );
            let stride = (6 * std::mem::size_of::<f32>()) as i32;
            self.gl.enable_vertex_attrib_array(self.position_location);
            self.gl.vertex_attrib_pointer_f32(
                self.position_location,
                2,
                glow::FLOAT,
                false,
                stride,
                0,
            );
            self.gl.enable_vertex_attrib_array(self.color_location);
            self.gl.vertex_attrib_pointer_f32(
                self.color_location,
                4,
                glow::FLOAT,
                false,
                stride,
                (2 * std::mem::size_of::<f32>()) as i32,
            );
            self.gl
                .uniform_matrix_4_f32_slice(Some(&self.projection_location), false, &projection);
            self.gl.viewport(
                0,
                0,
                ((draw_list.screen_size.x as f32) * draw_list.pixel_ratio) as i32,
                ((draw_list.screen_size.y as f32) * draw_list.pixel_ratio) as i32,
            );
            self.gl.clear_color(
                draw_list.bg_color.x,
                draw_list.bg_color.y,
                draw_list.bg_color.z,
                draw_list.bg_color.w,
            );
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            for cmd in &draw_list.cmds {
                let start = cmd.idx_offset;
                let count = cmd.num_of_elems * 3;
                self.gl.draw_elements(
                    glow::TRIANGLES,
                    count as i32,
                    glow::UNSIGNED_INT,
                    (start * std::mem::size_of::<u32>()) as i32,
                );
            }
            self.gl.flush();
        }
        Ok(())
    }
}
