
use super::Event;

use std::mem;

pub trait ContextBackend<V, U>
    where V: ShaderInputs,
          U: Uniforms
{
    fn init(&mut self);
    fn render(&self);
    fn get_events(&self) -> Vec<Event>;
    fn vertex_buffer(&mut self, vertexes: Vec<V>) -> Result<VertexBuffer, String>;
    fn program(&mut self,
               vssrc: &str,
               fssrc: &str,
               gssrc: Option<&str>,
               out: &str)
               -> Result<Program<U>, String>;
    fn draw(&self, vb: &VertexBuffer, program: &Program<U>, uniforms: &U);
}

pub struct Context<'a, V, U> {
    pub backend: Box<ContextBackend<V, U> + 'a>,
}

impl<'a, V, U> Context<'a, V, U>
    where V: ShaderInputs,
          U: Uniforms
{
    pub fn new<B>(backend: B) -> Context<'a, V, U>
        where B: ContextBackend<V, U> + 'a
    {
        Context { backend: Box::new(backend) }
    }
}

// TODO: BufferBackend
pub trait BufferBackend {}

pub trait ProgramBackend<U>
    where U: Uniforms
{
    fn draw(&self, vb: &VertexBuffer, uniforms: &U);
}

pub struct Program<'a, U> {
    pub backend: Box<ProgramBackend<U> + 'a>,
}

pub trait VertexBufferBackend {
    fn draw(&self);
    fn get_names<'a>(&self) -> Vec<&'a str>;
}

pub struct VertexBuffer<'a> {
    pub backend: Box<VertexBufferBackend + 'a>,
}

impl<'a> VertexBuffer<'a> {
    pub fn get_names(&self) -> Vec<&'a str> {
        self.backend.get_names()
    }
    pub fn draw(&self) {
        self.backend.draw();
    }
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
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
