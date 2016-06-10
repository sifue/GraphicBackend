
extern crate glutin;
extern crate gl;

pub use glutin::VirtualKeyCode as KeyCode;
pub use glutin::ElementState as KeyState;
pub use glutin::Event;
pub use glutin::MouseButton;

#[macro_use]
pub mod backend;
pub use backend::*;

pub mod opengl;
pub use opengl::OpenGL;
