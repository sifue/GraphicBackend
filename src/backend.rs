
use super::Event;

use std::mem;

pub trait Context {
    type Program;
    type VertexBuffer;
    fn init(&mut self);
    fn render(&self);
    fn get_events(&self) -> Vec<Event>;
    fn finish(&mut self);
    fn program(&mut self,
               vssrc: &str,
               fssrc: &str,
               gssrc: Option<&str>,
               out: &str)
               -> Result<Self::Program, String>;
    fn vertex_buffer(&mut self) -> Self::VertexBuffer;
}

pub trait Program {
    type Bind;
    type VertexBuffer: VertexBuffer;
    fn draw(&self, draw_type: DrawType, vb: &Self::VertexBuffer);
    fn get_bind(&self) -> Self::Bind;
}

pub trait Buffer {
    type Bind;
    fn get_buffer(&self) -> &Vec<ShaderInput>;
    fn get_bind(&self) -> Self::Bind;
    fn get_elem_size(&self) -> usize {
        input_size(&self.get_buffer()[0])
    }
}

pub trait VertexBuffer {
    type Buffer: Buffer;
    type Bind;
    type Program: Program;
    fn get_buffers(&self) -> &Vec<Self::Buffer>;
    fn get_binds(&self) -> Vec<<<Self as VertexBuffer>::Buffer as Buffer>::Bind> {
        self.get_buffers().iter().map(|b| b.get_bind()).collect()
    }
    fn get_names(&self) -> &Vec<String>;
    fn get_bind(&self) -> Self::Bind;
    fn add_input(mut self, name: &str, input: Vec<ShaderInput>) -> Self;
    fn build(mut self, program: &Self::Program) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub enum DrawType {
    Triangles,
    TriangleStrip,
}

#[derive(Clone, Debug)]
pub enum ShaderInput {
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
}

pub fn input_size(input: &ShaderInput) -> usize {
    use ShaderInput::*;
    match input {
        &Vec2(_, _) => 2 * mem::size_of::<f32>(),
        &Vec3(_, _, _) => 3 * mem::size_of::<f32>(),
    }
}

pub trait ShaderInputs {
    fn get_names<'a>() -> Vec<&'a str>;
    fn get_inputs(&self) -> Vec<ShaderInput>;
}

#[derive(Clone, Debug)]
pub enum Uniform {
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Matrix([[f32; 4]; 4]),
    Texture2D(usize, usize, ColorFormat /* &[u8] */),
}

pub trait Uniforms {
    fn get_names<'a>() -> Vec<&'a str>;
    fn get_params(&self) -> Vec<Uniform>;
}

#[derive(Clone, Copy, Debug)]
pub enum ColorFormat {
    RGB,
    RGBA,
}

macro_rules! impl_shader_param {
    ($from:ty, $to:ident, $($field:ident),+) => (
        impl $from {
            pub fn into_param(&self) -> ShaderParam {
                ShaderParam::$to($(self.$field),+)
            }
        }
    );
}

macro_rules! impl_vertex_params {
    ($name:ty, $($field:ident),+) => (
        impl ShaderInput for $name {
            fn get_binds(&self) -> Vec<ShaderParam> {
                vec![$(self.$field.into_param()),+]
            }
            fn get_names<'a>() -> Vec<&'a str> {
                vec![$(stringify!($field)),+]
            }
        }
    );
}

// pub struct Vertex {
//     pos: Vector3f,
//     color: Vector2f,
// }
//
// impl_shader_param!(Vector3f, Vec3, x, y, z);
// =>
// impl Vector3f {
//     pub fn into_param(&self) -> ShaderParam {
//         ShaderParam::Vec3(x, y, z)
//     }
// }

// impl_vertex_params!(Vertex, pos, color);
// // =>
// // impl VertexParams for Vertex {
// //     fn get_params(&self) -> Vec<ShaderParam> {
// //         vec![pos.into_param(), color.into_param()]
// //     }
// // }
