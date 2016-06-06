
use gl;

use glutin;
use glutin::{Window, WindowBuilder, GlRequest, GlProfile, Api};

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::ops::Drop;

use super::{ContextBackend, Context, ProgramBackend, Program, VertexBufferBackend, VertexBuffer,
            Uniforms, VertexParams, Event};

pub struct OpenGL {
    pub window: Window,
}

impl OpenGL {
    pub fn new() -> OpenGL {
        OpenGL {
            window: WindowBuilder::new()
                .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
                .with_gl_profile(GlProfile::Core)
                .build()
                .unwrap(),
        }
    }
}

impl<V, U> ContextBackend<V, U> for OpenGL
    where V: VertexParams + Copy + 'static,
          U: Uniforms
{
    fn init(&mut self) {
        unsafe {
            self.window.make_current().unwrap();
            gl::load_with(|s| self.window.get_proc_address(s) as *const _);
        };
    }
    fn render(&self) {
        self.window.swap_buffers().unwrap();
    }
    fn get_events(&self) -> Vec<Event> {
        let mut es = Vec::new();
        for e in self.window.poll_events() {
            es.push(e);
        }
        es
    }
    fn vertex_buffer(&mut self, vertexes: Vec<V>) -> Result<VertexBuffer, String> {
        let backend = try!(GLVertexBuffer::new(vertexes));
        Ok(VertexBuffer { backend: Box::new(backend) })
    }
    fn program(&mut self,
               vssrc: &str,
               fssrc: &str,
               gssrc: Option<&str>,
               out: &str)
               -> Result<Program<U>, String> {
        Ok(Program { backend: Box::new(try!(GLProgram::from_source(vssrc, fssrc, gssrc, out))) })
    }
    fn draw(&self, vb: &VertexBuffer, program: &Program<U>, uniforms: &U) {
        // program.set_inputs("", vb);
        // gl::VertexAttribPointer()
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
    vs: u32,
    fs: u32,
    gs: Option<u32>,
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
        let fs = try!(compile_shader(fssrc, gl::FRAGMENT_SHADER));
        attach_shader(program, fs);
        let gs = match gssrc {
            Some(s) => {
                let gsres = try!(compile_shader(s, gl::GEOMETRY_SHADER));
                attach_shader(program, gsres);
                Some(gsres)
            }
            None => None,
        };
        try!(link_program(program));
        unsafe {
            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0, CString::new(out).unwrap().as_ptr());
        }
        Ok(GLProgram {
            program: program,
            vs: vs,
            fs: fs,
            gs: gs,
        })
    }
}

impl<U> ProgramBackend<U> for GLProgram
    where U: Uniforms
{
    fn draw(&self, vb: &VertexBuffer, uniforms: &U) {
        unimplemented!()
    }
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.vs);
            gl::DeleteShader(self.fs);
            match self.gs {
                Some(gs) => gl::DeleteShader(gs),
                None => (),
            }
        }
    }
}

pub struct GLVertexBuffer<V> {
    buffer: Vec<V>,
    vbos: Vec<u32>,
}

impl<V> GLVertexBuffer<V>
    where V: VertexParams
{
    pub fn new(vertexes: Vec<V>) -> Result<GLVertexBuffer<V>, String> {
        let mut buffers = Vec::new();
        for vert in vertexes.iter() {
            buffers.push(Vec::new());
            let params = vert.get_params();
            for i in 0..params.len() {
                buffers[i].push(params[i]);
            }
        }
        let mut vbos = Vec::new();
        for buffer in buffers.iter() {
            let mut vbo: u32 = 0;
            unsafe {
                gl::GenBuffers(1, &mut vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(gl::ARRAY_BUFFER,
                               (buffer.len() * mem::size_of_val(&buffer[0])) as isize,
                               mem::transmute(&buffer[0]),
                               gl::STATIC_DRAW);
            }
            vbos.push(vbo);
        }
        Ok(GLVertexBuffer {
            buffer: vertexes,
            vbos: vbos,
        })
    }
}

impl<V> VertexBufferBackend for GLVertexBuffer<V>
    where V: VertexParams
{
    fn draw(&self) {
        unimplemented!()
    }
}
