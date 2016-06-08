
extern crate graphic_backend;

static VS_SRC: &'static str = r#"
    #version 150
    in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
"#;
static FS_SRC: &'static str = r#"
    #version 150
    out vec4 out_color;
    void main() {
        out_color = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

use graphic_backend::*;

fn main() {
    use graphic_backend::InputBuffer::*;

    let mut context = OpenGL::new();
    context.init();
    let program = context.program(VS_SRC, FS_SRC, None, "out_color").unwrap();
    let vertexes = Vec3(vec![-1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, 0.0]);
    let vb = context.vertex_buffer().add_input("position", vertexes).build(&program);

    loop {
        context.draw(&program, DrawType::Triangles, &vb);
        context.finish();
    }
}
