use wasm_bindgen::prelude::*;
use golem::Dimension::*;
use golem::*;
use super::cad::DrawList;

#[wasm_bindgen]
pub struct GolemBackend {
    golem_ctx: Context,
    shader: ShaderProgram,
    vb: VertexBuffer,
    eb: ElementBuffer,
}

#[wasm_bindgen]
impl GolemBackend {
    #[wasm_bindgen(constructor)]
    pub fn from_webgl(webgl_context: web_sys::WebGlRenderingContext) -> Self {
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let golem_context = golem::Context::from_glow(glow_context).unwrap();
        GolemBackend::new(golem_context).unwrap()
    }
}

impl GolemBackend {
    pub fn new(golem_ctx: Context) -> Result<Self, GolemError> {
        let mut shader = ShaderProgram::new(
            &golem_ctx,
            ShaderDescription {
                vertex_input: &[
                    Attribute::new("vert_position", AttributeType::Vector(D2)),
                    Attribute::new("vert_color", AttributeType::Vector(D4)),
                    //Attribute::new("vert_uv", AttributeType::Vector(D2)),
                ],
                fragment_input: &[
                    Attribute::new("frag_color", AttributeType::Vector(D4)),
                    //Attribute::new("frag_uv", AttributeType::Vector(D2)),
                ],
                uniforms: &[
                    Uniform::new("projection", UniformType::Matrix(D4)),
                    //Uniform::new("texture", UniformType::Sampler2D),
                ],
                vertex_shader: r#" void main() {
                    gl_Position = projection * vec4(vert_position, 0, 1);
                    frag_color = vert_color;
                }"#,
                fragment_shader: r#" void main() {
                    gl_FragColor = frag_color;
                }"#,
            },
        )?;
        let vb = VertexBuffer::new(&golem_ctx)?;
        let eb = ElementBuffer::new(&golem_ctx)?;
        shader.bind();
        Ok(Self {
            golem_ctx,
            shader,
            vb,
            eb,
        })
    }

    pub fn draw(&mut self, draw_list: &DrawList) -> Result<(), GolemError> {
        let w = draw_list.screen_size.x as f32;
        let h = draw_list.screen_size.y as f32;
        let scale = draw_list.scale;
        let translate = draw_list.translate;
        let sx = (2. / w) * scale;
        let sy = (2. / h) * scale;
        let npx = 2. * translate.x / w + 1. / w;
        let npy = -2. * translate.y / h + 1. / h;
        let projection = UniformValue::Matrix4([
            sx, 0., 0., 0.,
            0., -sy, 0., 0.,
            0., 0., -1., 0.,
            npx - 1., npy + 1., 0., 1.,
        ]);
        let vertices = draw_list.vertices();
        let indices = draw_list.indices();
        self.vb.set_data(&vertices);
        self.eb.set_data(&indices);
        self.shader.prepare_draw(&self.vb, &self.eb)?;
        self.shader.set_uniform("projection", projection)?;
        self.golem_ctx.set_viewport(0, 0, draw_list.screen_size.x, draw_list.screen_size.y);
        self.golem_ctx.set_clear_color(draw_list.bg_color.x, draw_list.bg_color.y, draw_list.bg_color.z, draw_list.bg_color.w);
        self.golem_ctx.clear();
        unsafe {
            self.shader.draw_prepared(0..indices.len(), GeometryMode::Triangles);
        }
        Ok(())
    }
}
