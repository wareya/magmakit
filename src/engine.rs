use std::collections::HashMap;
use std::rc::Rc;

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
    
    program_index_counter : u64,
    programs: HashMap<u64, Rc<glium::Program>>,
    
    draw_events: Vec<DrawEvent>,
    
    display: glium::Display,
    
    vertex_buffer: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,
    default_program: Rc<glium::Program>,
    
    current_program: Rc<glium::Program>,
    
    text_system: TextSystem,
    
}

impl Engine {
    pub (crate) fn load(display : glium::Display, program_path : String) -> Engine
    {
        let glprogram = Rc::new(Engine::build_glprogram(&display, &program_path));
        let (vertex_buffer, indices) = Engine::build_vertex_buffer(&display);
        let text_system = TextSystem::new(&display);
        Engine{program_path, input_handler : InputHandler::new(), sprite_index_counter : 1, sprites : HashMap::new(), program_index_counter : 1, programs : HashMap::new(), draw_events : Vec::new(), display, vertex_buffer, indices, default_program : Rc::clone(&glprogram), current_program : Rc::clone(&glprogram), text_system}
    }
}
