
extern crate graphic_backend;

static VS_SRC: &'static str = r#""#;
static FS_SRC: &'static str = r#""#;

use graphic_backend::*;

fn main() {
    use ShaderInput::*;

    let mut context = OpenGL::new();
    let program = context.program(VS_SRC, FS_SRC, None, "out_color");
    let vertexes = vec![Vec3(-1.0, -1.0, 0.0), Vec3(0.0, 1.0, 0.0), Vec3(1.0, -1.0, 0.0)];
    let vb = context.vertex_buffer().add_input("position", vertexes).build(&program);
}
