
use super::Event;

pub trait ContextBackend<V, U>
    where V: VertexParams + Copy,
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
    fn draw(&self, vb: &VertexBuffer, program: &Program<U>, uniforms: &U)
        where V: VertexParams,
              U: Uniforms;
}

pub struct Context<'a, V, U> {
    pub backend: Box<ContextBackend<V, U> + 'a>,
}

impl<'a, V, U> Context<'a, V, U>
    where V: VertexParams,
          U: Uniforms
{
    pub fn new<B>(backend: B) -> Context<'a, V, U>
        where B: ContextBackend<V, U> + 'a
    {
        Context { backend: Box::new(backend) }
    }
}

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
}

pub struct VertexBuffer<'a> {
    pub backend: Box<VertexBufferBackend + 'a>,
}

pub trait Uniforms {
    fn get_uniforms(&self) -> Vec<ShaderParam>;
    fn get_names<'a>() -> Vec<&'a str>;
}

pub trait VertexParams: Copy {
    fn get_params(&self) -> Vec<ShaderParam>;
    fn get_names<'a>() -> Vec<&'a str>;
}

#[derive(Clone, Copy, Debug)]
pub enum ColorFormat {
    RGB,
    RGBA,
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderParam {
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Matrix([[f32; 4]; 4]),
    Texture2D(usize, usize, ColorFormat /* &[u8] */),
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
        impl VertexParams for $name {
            fn get_params(&self) -> Vec<ShaderParam> {
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
