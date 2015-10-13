#![feature(slice_patterns)] 
extern crate num;
#[macro_use]
extern crate glium;
extern crate nalgebra;
use nalgebra::*;
use glium::{DisplayBuild, Surface};
use num::traits::Zero;
use glium::glutin::Event::*;
use glium::glutin::ElementState::*;
use glium::glutin::MouseScrollDelta::PixelDelta;
use glium::glutin::MouseButton::*;

fn main() {
  let display = glium::glutin::WindowBuilder::new()
                  .with_vsync()
                  .build_glium().unwrap();

  #[derive(Copy, Clone)]
  struct Vertex {position: [f32; 2]}
  implement_vertex!(Vertex, position);

  let vertex_shader_src = r#"
      #version 140
      uniform mat2 projection;
      uniform vec2 translation;
      in vec2 position;
      void main() {
          gl_Position = vec4(projection * (position + translation), 0.0, 1.0);
      }
  "#;

  let fragment_shader_src = r#"
      #version 140

      out vec4 color;

      void main() {
          color = vec4(0.0, 0.0, 0.0, 1.0);
      }
  "#;
  
  let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);

  let params = glium::DrawParameters {
    line_width: Some(2.0),
    .. Default::default()
  };

  let program = glium::Program::from_source(
    &display, 
    vertex_shader_src, 
    fragment_shader_src, 
    None).unwrap();


  let mut current_line : Vec<Vertex> = Vec::new();
  let mut commited_lines : Vec<glium::VertexBuffer<Vertex>> = Vec::new();

  #[derive(Copy, Clone)]
  enum State {
    None,
    Drawing,
    Dragging(Vec2<f32>)
  }

  let mut state = State::None;
  
  let mut scale = 1.0;
  let mut translation = Vec2::zero();
  let mut mouse_last = Vec2::zero();

  loop {

    let (frame_dx, frame_dy) = display.get_framebuffer_dimensions();
    let (frame_dx, frame_dy) = (frame_dx as f32, frame_dy as f32);
    
    let projection
      = Mat2::new(2.0/frame_dx, 0.0,
                  0.          , 2.0/frame_dy);
                  
    for event in display.poll_events() {
      match (event, state) {
      
        (MouseWheel(PixelDelta(_, dy)), _)  => {
          if dy > 0.0 {
            scale /= 0.99;
          } else { 
            scale *= 0.99; 
          }
          translation = -mouse_last + mouse_last/scale;
        },

        (MouseInput(Pressed,Left), State::None) => {
          state = State::Drawing;
        },
        
        (MouseInput(Pressed,Middle), State::None) => {
            state = State::Dragging(mouse_last);
        },

        (MouseInput(Released,Left), State::Drawing) => {
            commited_lines.push(
              glium::VertexBuffer::immutable(&display, &current_line).unwrap());
            current_line.clear();
            state = State::None;
        },
        
        (MouseInput(Released,Middle), State::Dragging(_)) => {
            state = State::None;
        },

        (MouseMoved((mouse_x, mouse_y)), state) => {
          let screen 
            = Vec2::new(
                2.0*(mouse_x as f32)/frame_dx - 1.0, 
               -2.0*(mouse_y as f32)/frame_dy + 1.0);
               
          let world  = projection.inv().unwrap() 
                        * (1.0/scale) 
                        * screen 
                        - translation;

          match state {
            State::Drawing => {
              current_line.push(Vertex{position: [world.x, world.y]});
            },
            State::Dragging(drag_start) => {
              translation.x +=  world.x - drag_start.x;
              translation.y +=  world.y - drag_start.y;
            },
            _ => {}
          }
          
          mouse_last = world;
        },
        
        (Closed, _) => return,
        _ => ()
      }
    }

    let uniforms = uniform!(
      projection : projection * scale,
      translation: translation,
    );

    let mut target = display.draw();
    target.clear_color(1.0, 1.0, 1.0, 1.0);

    for line in &commited_lines[..] {
      target.draw(
        line,
        &indices,
        &program,
        &uniforms,
        &params).unwrap();
    }
    
    if let State::Drawing = state {
        target.draw(
          &glium::VertexBuffer::new(&display, &current_line).unwrap(),
          &indices,
          &program,
          &uniforms,
          &Default::default()).unwrap();
    }

    target.finish().unwrap();
  }
}
