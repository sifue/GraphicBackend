
use super::Event;

use std::mem;

pub trait Facade {
    fn program(&self,
               vssrc: &str,
               fssrc: &str,
               gssrc: Option<&str>,
               out: &str)
               -> Result<Box<Program>, String>;
    fn vertex_buffer(&self) -> Box<VertexBufferBuilder>;
    fn frame(&self) -> Box<Frame>;
    fn texture2d(&self,
                 format: ColorFormat,
                 width: u32,
                 height: u32,
                 data: Vec<u8>)
                 -> Box<Texture2D>;
}

macro_rules! impl_facade {
    ($name:ident, $selfcontext:ident, {
        Context => $context:ident,
        Frame => $frame:ident,
        Program => $program:ident,
        VertexBufferBuilder => $vbb:ident,
        Texture2D => $tex2d:ident,
    }) => (
        impl Facade for $name {
            fn program(&self,
                       vssrc: &str,
                       fssrc: &str,
                       gssrc: Option<&str>,
                       out: &str)
                       -> Result<Box<Program>, String> {
                Ok(Box::new(try!($program::from_source(vssrc, fssrc, gssrc, out))))
            }
            fn vertex_buffer(&self) -> Box<VertexBufferBuilder> {
                Box::new($vbb::new())
            }
            fn frame(&self) -> Box<Frame> {
                Box::new($frame::new(self.$selfcontext.clone()))
            }
            fn texture2d(&self, format: ColorFormat, width: u32, height: u32, data: Vec<u8>) -> Box<Texture2D> {
                Box::new($tex2d::new(format, width, height, data))
            }
        }
    );
}

pub trait Context {
    fn get_events(&self) -> Vec<Event>;
    fn finish(&self);
}

pub trait Frame {
    fn draw(&mut self,
            program: &Box<Program>,
            draw_type: DrawType,
            vb: &Box<VertexBuffer>,
            uniforms: &Uniforms<u32>);
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32);
    fn finish(self: Box<Self>);
}

pub trait Program {
    fn draw(&self, draw_type: DrawType, vb: &Box<VertexBuffer>, uniforms: &Uniforms<u32>);
    fn get_bind(&self) -> u32;
}

pub trait Buffer {
    fn get_buffer(&self) -> &InputBuffer;
    fn get_bind(&self) -> u32;
    fn elem_len(&self) -> usize {
        self.get_buffer().elem_len()
    }
    fn len(&self) -> usize {
        self.get_buffer().len()
    }
}

pub trait VertexBuffer {
    fn get_buffers(&self) -> &Vec<Box<Buffer>>;
    fn get_binds(&self) -> Vec<u32> {
        self.get_buffers().iter().map(|b| b.get_bind()).collect()
    }
    fn get_names(&self) -> &Vec<String>;
    fn get_bind(&self) -> u32;
    fn len(&self) -> usize {
        self.get_buffers()[0].len()
    }
}

pub trait VertexBufferBuilder {
    fn add_input(mut self: Box<Self>, name: &str, input: InputBuffer) -> Box<VertexBufferBuilder>;
    fn build(self: Box<Self>, program: &Box<Program>) -> Box<VertexBuffer>;
}

#[derive(Clone, Copy, Debug)]
pub enum DrawType {
    Triangles,
    TriangleStrip,
}

#[derive(Clone, Debug)]
pub enum InputBuffer {
    Vec2(Vec<f32>),
    Vec3(Vec<f32>),
}

impl InputBuffer {
    pub fn len(&self) -> usize {
        use InputBuffer::*;
        match self {
            &Vec2(ref v) => v.len() / 2,
            &Vec3(ref v) => v.len() / 3,
        }
    }
    pub fn elem_len(&self) -> usize {
        use InputBuffer::*;
        match self {
            &Vec2(..) => 2,
            &Vec3(..) => 3,
        }
    }
    pub fn buffer_size(&self) -> usize {
        use InputBuffer::*;
        match self {
            &Vec2(ref v) => v.len() * mem::size_of::<f32>(),
            &Vec3(ref v) => v.len() * mem::size_of::<f32>(),
        }
    }
    pub fn as_slice(&self) -> &[f32] {
        use InputBuffer::*;
        match self {
            &Vec2(ref raw) => raw.as_slice(),
            &Vec3(ref raw) => raw.as_slice(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColorFormat {
    RGB,
    RGBA,
}

impl ColorFormat {
    pub fn size(self) -> usize {
        use ColorFormat::*;
        match self {
            RGB => 3,
            RGBA => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Uniform<T> {
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Matrix([[f32; 4]; 4]),
    Texture2D(T),
}

pub struct Uniforms<T> {
    pub names: Vec<String>,
    pub uniforms: Vec<Uniform<T>>,
}

impl<T> Uniforms<T> {
    pub fn new() -> Uniforms<T> {
        Uniforms {
            names: Vec::new(),
            uniforms: Vec::new(),
        }
    }
    pub fn add_uniform(&mut self, name: &str, uniform: Uniform<T>) {
        self.names.push(String::from(name));
        self.uniforms.push(uniform);
    }
}

pub trait Texture2D {
    fn get_bind(&self) -> u32;
    fn as_uniform(&self) -> Uniform<u32> {
        Uniform::Texture2D(self.get_bind())
    }
}

#[macro_export]
macro_rules! uniforms {
    ($($name:ident : $val:expr),*) => (
        {
            let mut uniforms = Uniforms::new();
            $(uniforms.add_uniform(stringify!($name), $val);),*
            uniforms
        }
    );
}

// pub trait ShaderInputs {
//     fn get_names<'a>() -> Vec<&'a str>;
//     fn get_inputs(&self) -> Vec<ShaderInput>;
// }
//
// #[derive(Clone, Debug)]
// pub enum Uniform {
//     Vec2(f32, f32),
//     Vec3(f32, f32, f32),
//     Matrix([[f32; 4]; 4]),
//     Texture2D(usize, usize, ColorFormat /* &[u8] */),
// }
//
// pub trait Uniforms {
//     fn get_names<'a>() -> Vec<&'a str>;
//     fn get_params(&self) -> Vec<Uniform>;
// }
//

// macro_rules! impl_shader_param {
//     ($from:ty, $to:ident, $($field:ident),+) => (
//         impl $from {
//             pub fn into_param(&self) -> ShaderParam {
//                 ShaderParam::$to($(self.$field),+)
//             }
//         }
//     );
// }
//
// macro_rules! impl_vertex_params {
//     ($name:ty, $($field:ident),+) => (
//         impl ShaderInput for $name {
//             fn get_binds(&self) -> Vec<ShaderParam> {
//                 vec![$(self.$field.into_param()),+]
//             }
//             fn get_names<'a>() -> Vec<&'a str> {
//                 vec![$(stringify!($field)),+]
//             }
//         }
//     );
// }

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
