use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::input::InputHandler;
use crate::{open_file, load_string};

pub (crate) mod bindings;
pub (crate) mod render;

use self::render::*;

pub (crate) struct Engine {
    program_path: String,
    
    display: glium::Display,
    
    pub (crate) input_handler: InputHandler,
    
    sprite_index_counter: u64,
    sprites: HashMap<u64, SpriteSheet>,
    
    program_index_counter : u64,
    programs: HashMap<u64, Rc<glium::Program>>,
    
    draw_target: Option<glium::Frame>,
    default_surface: Option<Surface>,
    draw_w: u32,
    draw_h: u32,
    matrix_view: [[f32; 4]; 4],
    
    surface_index_counter : u64,
    surfaces: HashMap<u64, Surface>,
    surface_target: Vec<u64>,
    
    vertex_buffer: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,
    current_program: Rc<glium::Program>,
    
    default_program: Rc<glium::Program>,
    
    text_system: RefCell<TextSystem>,
    
}

impl Engine {
    pub (crate) fn load(display : glium::Display, program_path : String) -> Engine
    {
        let glprogram = Rc::new(Engine::build_glprogram(&display, &program_path));
        let (vertex_buffer, indices) = Engine::build_vertex_buffer(&display);
        let text_system = TextSystem::new(&display);
        Engine
        {
            program_path,
            
            display,
            
            input_handler : InputHandler::new(),
            
            sprite_index_counter : 1,
            sprites : HashMap::new(),
            
            program_index_counter : 1,
            programs : HashMap::new(),
            
            draw_target : None,
            default_surface : None,
            draw_w : 0,
            draw_h : 0,
            matrix_view : [[0.0; 4]; 4],
            
            surface_index_counter : 1,
            surfaces : HashMap::new(),
            surface_target : Vec::new(),
            
            vertex_buffer,
            indices,
            current_program : Rc::clone(&glprogram),
            
            default_program : Rc::clone(&glprogram),
            
            text_system : RefCell::new(text_system)
        }
    }
}
