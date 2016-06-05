
use std::mem;

use gl;
use glutin;
use glutin::{Window, WindowBuilder, GlRequest, GlProfile, Api};

use super::{Backend, Program, Uniforms, VertexParams, VertexBuffer, Event};

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

impl Backend<u32> for OpenGL {
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
    fn vertex_buffer<V>(&mut self, vertexes: Vec<V>) -> VertexBuffer<V, u32>
        where V: VertexParams
    {
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
        VertexBuffer {
            buffer: vertexes,
            bindings: vbos,
        }
    }
    fn draw<V, P, U>(&self, vb: VertexBuffer<V, u32>, program: P, uniforms: U)
        where V: VertexParams,
              P: Program<u32>,
              U: Uniforms
    {
        // program.set_inputs("", vb);
        // gl::VertexAttribPointer()
    }
}
