
#[macro_use]
extern crate graphic_backend;
extern crate image;

use std::fs::File;
use std::path::Path;
use image::GenericImage;

use graphic_backend::*;

static VS_SRC: &'static str = r#"
    #version 150
    in vec3 position;
    in vec2 coord;
    out vec2 coord0;
    void main() {
        coord0 = coord;
        gl_Position = vec4(position, 1.0);
    }
"#;
static FS_SRC: &'static str = r#"
    #version 150
    in vec2 coord0;
    out vec4 out_color;
    uniform sampler2D tex;
    void main() {
        // out_color = vec4(1.0, 1.0, 1.0, 1.0);
        out_color = texture2D(tex, coord0);
    }
"#;

fn main() {
    use graphic_backend::InputBuffer::*;

    let mut facade = OpenGL::new();
    let program = facade.program(VS_SRC, FS_SRC, None, "out_color").unwrap();
    let vertexes = Vec3(vec![-1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, 0.0]);
    let coords = Vec2(vec![-1.0, -1.0, 0.0, 1.0, 1.0, -1.0]);
    let vb = facade.vertex_buffer()
        .add_input("position", vertexes)
        .add_input("coord", coords)
        .build(&program);
    // let mut uniforms = Uniforms::new();

    let img = image::open(&Path::new("resource/denim.png")).unwrap();
    let (width, height) = img.dimensions();
    let tex = facade.texture2d(ColorFormat::RGBA, width, height, img.to_rgba().into_raw());
    // uniforms.add_uniform("tex", tex.as_uniform());

    let uniforms = uniforms! {
        tex: tex.as_uniform()
    };

    loop {
        let mut frame = facade.frame();
        frame.draw(&program, DrawType::Triangles, &vb, &uniforms);
        frame.finish();
    }
}
