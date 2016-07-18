
use gl;

use glutin::{Window, WindowBuilder, GlRequest, GlProfile, Api};

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::ops::Drop;
use std::rc::Rc;

use super::backend::*;
use super::event::*;

pub struct OpenGL {
    context: Rc<GLContext>,
}

impl OpenGL {
    pub fn new() -> OpenGL {
        OpenGL { context: Rc::new(GLContext::new()) }
    }
}

impl_facade!(OpenGL, context, {
    Context => GLContext,
    Frame => GLFrame,
    Program => GLProgram,
    VertexBufferBuilder => GLVertexBufferBuilder,
    Texture2D => GLTexture2D,
});

pub struct GLContext {
    pub window: Window,
}

impl GLContext {
    pub fn new() -> GLContext {
        let window = WindowBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
            .with_gl_profile(GlProfile::Core)
            .build()
            .unwrap();
        unsafe {
            window.make_current().unwrap();
            gl::load_with(|s| window.get_proc_address(s) as *const _);
        }
        GLContext { window: window }
    }
}

impl Context for GLContext {
    fn get_events(&self) -> Vec<Event> {
        let mut es = Vec::new();
        for e in self.window.poll_events() {
            es.push(e);
        }
        es
    }
    fn finish(&self) {
        self.window.swap_buffers().unwrap();
    }
}

pub struct GLFrame {
    context: Rc<GLContext>,
}

impl GLFrame {
    pub fn new(context: Rc<GLContext>) -> GLFrame {
        GLFrame { context: context }
    }
}

impl Frame for GLFrame {
    fn draw(&mut self,
            program: &BProgram,
            draw_type: DrawType,
            vb: &BVertexBuffer,
            uniforms: &Uniforms<u32>) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        program.draw(draw_type, vb, uniforms);
    }
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }
    fn finish(self: Box<Self>) {
        self.context.finish();
    }
}

pub fn compile_shader(src: &str, ty: GLenum) -> Result<u32, String> {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::null_mut(),
                                 buf.as_mut_ptr() as *mut GLchar);
            return Err(String::from(str::from_utf8(&buf)
                .ok()
                .expect("ShaderInfoLog not valid utf8")));
        }
    }
    Ok(shader)
}

pub fn create_program() -> u32 {
    unsafe { gl::CreateProgram() }
}

pub fn attach_shader(program: u32, shader: u32) {
    unsafe {
        gl::AttachShader(program, shader);
    }
}

pub fn link_program(program: u32) -> Result<u32, String> {
    unsafe {
        gl::LinkProgram(program);
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(program,
                                  len,
                                  ptr::null_mut(),
                                  buf.as_mut_ptr() as *mut GLchar);
            return Err(String::from(str::from_utf8(&buf)
                .ok()
                .expect("ProgramInfoLog not valid utf8")));
        }
    }
    Ok(program)
}

pub struct GLProgram {
    program: u32,
}

impl GLProgram {
    fn from_source(vssrc: &str,
                   fssrc: &str,
                   gssrc: Option<&str>,
                   out: &str)
                   -> Result<GLProgram, String> {
        let program = create_program();

        let vs = try!(compile_shader(vssrc, gl::VERTEX_SHADER));
        attach_shader(program, vs);
        unsafe {
            gl::DeleteShader(vs);
        }

        let fs = try!(compile_shader(fssrc, gl::FRAGMENT_SHADER));
        attach_shader(program, fs);
        unsafe {
            gl::DeleteShader(fs);
        }

        match gssrc {
            Some(s) => {
                let gs = try!(compile_shader(s, gl::GEOMETRY_SHADER));
                attach_shader(program, gs);
                unsafe { gl::DeleteShader(gs) }
            }
            None => (),
        }

        unsafe {
            // gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0, CString::new(out).unwrap().as_ptr());
        }

        try!(link_program(program));

        Ok(GLProgram { program: program })
    }
}

impl Program for GLProgram {
    fn draw(&self, draw_type: DrawType, vb: &BVertexBuffer, uniforms: &Uniforms<u32>) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(vb.get_bind());
            gl::DrawArrays(draw_type_to_gl_type(draw_type), 0, vb.len() as i32);
        }
        let names = &uniforms.names;
        let params = &uniforms.uniforms;
        let mut texid = 0;
        for (name, param) in names.iter().zip(params.iter()) {
            let loc: i32 = 0;
            unsafe {
                gl::GetUniformLocation(self.program,
                                       CString::new(name.clone().into_bytes()).unwrap().as_ptr());
            }
            set_uniform_value(loc, param, &mut texid);
        }
    }
    fn get_bind(&self) -> u32 {
        self.program
    }
}

pub fn draw_type_to_gl_type(t: DrawType) -> GLenum {
    use DrawType::*;
    match t {
        Triangles => gl::TRIANGLES,
        TriangleStrip => gl::TRIANGLE_STRIP,
        // _ => panic!("{:?}: this type is still not supported draw type.", t),
    }
}

pub fn set_uniform_value(loc: i32, val: &Uniform<u32>, texid: &mut u32) {
    use Uniform::*;
    match val {
        &Vec2(x, y) => unsafe {
            gl::Uniform2f(loc, x, y);
        },
        &Vec3(x, y, z) => unsafe {
            gl::Uniform3f(loc, x, y, z);
        },
        &Matrix(m) => unsafe {
            gl::UniformMatrix4fv(loc, 1, gl::TRUE, mem::transmute(&m[0]));
        },
        &Texture2D(b) => unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + *texid); // u32 to i32 ?
            gl::BindTexture(gl::TEXTURE_2D, b);
            gl::Uniform1i(loc, *texid as i32);
            *texid += 1;
        },
        // _ => {
        //     panic!("{:?}: this type is still not supported parameter type.",
        //            val);
        // }
    }
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

pub struct GLBuffer {
    buffer: InputBuffer,
    bind: u32,
}

impl GLBuffer {
    pub fn new(buffer: InputBuffer) -> GLBuffer {
        let mut bind: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut bind);
            gl::BindBuffer(gl::ARRAY_BUFFER, bind);
            gl::BufferData(gl::ARRAY_BUFFER,
                           buffer.buffer_size() as isize,
                           mem::transmute(&buffer.as_slice()[0]),
                           gl::STATIC_DRAW);
        }
        GLBuffer {
            buffer: buffer,
            bind: bind,
        }
    }
}

impl Buffer for GLBuffer {
    fn get_buffer(&self) -> &InputBuffer {
        &self.buffer
    }
    fn get_bind(&self) -> u32 {
        self.bind
    }
}

impl Drop for GLBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.bind);
        }
    }
}

pub struct GLVertexBuffer {
    names: Vec<String>,
    buffers: Vec<BBuffer>,
    vao: u32,
}

impl GLVertexBuffer {
    pub fn new(names: Vec<String>, buffers: Vec<BBuffer>, vao: u32) -> GLVertexBuffer {
        GLVertexBuffer {
            names: names,
            buffers: buffers,
            vao: vao,
        }
    }
}

impl VertexBuffer for GLVertexBuffer {
    fn get_buffers(&self) -> &Vec<BBuffer> {
        &self.buffers
    }
    fn get_names(&self) -> &Vec<String> {
        &self.names
    }
    fn get_bind(&self) -> u32 {
        self.vao
    }
}

pub struct GLVertexBufferBuilder {
    names: Vec<String>,
    buffers: Vec<BBuffer>,
}

impl GLVertexBufferBuilder {
    pub fn new() -> GLVertexBufferBuilder {
        GLVertexBufferBuilder {
            names: Vec::new(),
            buffers: Vec::new(),
        }
    }
}
impl VertexBufferBuilder for GLVertexBufferBuilder {
    fn add_input(mut self, name: &str, input: InputBuffer) -> GLVertexBufferBuilder {
        self.names.push(String::from(name));
        self.buffers.push(Box::new(GLBuffer::new(input)));
        self
    }
    fn build(self, program: &BProgram) -> BVertexBuffer {
        let mut vao: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            for (name, buffer) in self.names.iter().zip(self.buffers.iter()) {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer.get_bind());
                let loc: u32 =
                    gl::GetAttribLocation(program.get_bind(),
                                          CString::new((&name).as_bytes())
                                              .unwrap()
                                              .as_ptr()) as u32;

                gl::VertexAttribPointer(loc,
                                        buffer.elem_len() as i32,
                                        gl::FLOAT,
                                        gl::FALSE,
                                        0,
                                        ptr::null());
                gl::EnableVertexAttribArray(loc);
            }
        }
        Box::new(GLVertexBuffer::new(self.names, self.buffers, vao))
    }
}

pub struct GLTexture2D {
    format: ColorFormat,
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    bind: u32,
}

pub fn color_format_to_gl_type(format: ColorFormat) -> GLenum {
    use ColorFormat::*;
    match format {
        RGB => gl::RGB,
        RGBA => gl::RGBA,
    }
}

impl GLTexture2D {
    pub fn new(format: ColorFormat, width: u32, height: u32, data: Vec<u8>) -> GLTexture2D {
        let mut bind: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut bind);
            gl::BindTexture(gl::TEXTURE_2D, bind);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           color_format_to_gl_type(format) as i32,
                           width as i32,
                           height as i32,
                           0,
                           color_format_to_gl_type(format),
                           gl::UNSIGNED_BYTE,
                           mem::transmute(&data.as_slice()[0]));
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        }
        GLTexture2D {
            format: format,
            width: width,
            height: height,
            buffer: data,
            bind: bind,
        }
    }
}

impl Texture2D for GLTexture2D {
    fn get_bind(&self) -> u32 {
        self.bind
    }
}
