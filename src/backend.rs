
use super::Event;

pub trait Backend<B> {
    fn init(&mut self);
    fn render(&self);
    fn get_events(&self) -> Vec<Event>;
    fn vertex_buffer<V>(&mut self, vertexes: Vec<V>) -> VertexBuffer<V, B> where V: VertexParams;
    fn program(&mut self,
               vssrc: &str,
               fssrc: &str,
               gssrc: Option<&str>,
               out: &str)
               -> Result<Program<u32>, String>;
    fn draw<V, U>(&self, vb: VertexBuffer<V, u32>, program: Program<B>, uniforms: U)
        where V: VertexParams,
              U: Uniforms;
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

pub struct Program<B> {
    pub binding: B,
}

pub struct VertexBuffer<V, B>
    where V: VertexParams + Copy
{
    pub buffer: Vec<V>,
    pub bindings: Vec<B>,
}

pub trait Uniforms {
    fn get_uniforms(&self) -> Vec<ShaderParam>;
    fn get_names<'a>() -> Vec<&'a str>;
}

pub trait VertexParams: Copy {
    fn get_params(&self) -> Vec<ShaderParam>;
    fn get_names<'a>() -> Vec<&'a str>;
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
