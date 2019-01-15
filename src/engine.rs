use std::collections::HashMap;

use crate::input::InputHandler;
use crate::{open_file, load_string};

pub (crate) mod bindings;
pub (crate) mod render;

use self::render::*;

pub (crate) struct Engine {
    program_path: String,
    
    pub (crate) input_handler: InputHandler,
    
    sprite_index_counter: u64,
    sprites: HashMap<u64, SpriteSheet>,
    draw_events: Vec<DrawEvent>,
    
    display: glium::Display,
    
    vertex_buffer: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,
    glprogram: glium::Program
}

impl Engine {
    pub (crate) fn load(display : glium::Display, program_path : String) -> Engine
    {
        let glprogram = Engine::build_glprogram(&display, &program_path);
        let (vertex_buffer, indices) = Engine::build_vertex_buffer(&display);
        Engine{program_path, input_handler : InputHandler::new(), sprite_index_counter : 1, sprites : HashMap::new(), draw_events : Vec::new(), display, vertex_buffer, indices, glprogram}
    }
}
