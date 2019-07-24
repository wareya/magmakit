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
    
    target_frametime: f64, // seconds
    framelimiter_reference: Option<std::time::Instant>,
    framelimiter_count: u64,
    framelimiter_delta: f64,
    framelimiter_check_desync: bool,
    recent_deltas: Vec<f64>,
}

fn duration_to_secs(duration : &std::time::Duration) -> f64
{
    duration.as_secs() as f64 + duration.subsec_nanos() as f64/1_000_000_000.0
}
fn duration_from_secs(secs : f64) -> std::time::Duration
{
    std::time::Duration::new(secs.trunc() as u64, (secs.fract() * 1_000_000_000.0).trunc() as u32)
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
            
            text_system : RefCell::new(text_system),
            
            target_frametime : 0.008,
            framelimiter_reference : None,
            framelimiter_count: 0,
            framelimiter_delta: 0.0,
            framelimiter_check_desync: false,
            recent_deltas: Vec::new(),
        }
    }
    pub (crate) fn unsafe_check_global_cursor_position(&mut self)
    {
        if cfg!(windows)
        {
            use winapi::um::winuser::GetCursorPos;
            use winapi::shared::windef::POINT;
            let (valid, cursor_x, cursor_y);
            unsafe
            {
                let mut cursor_pos : POINT = std::mem::zeroed();
                valid = GetCursorPos(&mut cursor_pos);
                cursor_x = cursor_pos.x;
                cursor_y = cursor_pos.y;
            }
            if valid != 0
            {
                let factor = self.display.gl_window().window().get_hidpi_factor();
                let offset = self.display.gl_window().window().get_inner_position().unwrap();
                let new_x = factor*cursor_x as f64 - offset.x;
                let new_y = factor*cursor_y as f64 - offset.y;
                self.input_handler.mouse_pos = (new_x, new_y);
            }
        }
    }
    pub (crate) fn check_init_framerate_limiter(&mut self)
    {
        if self.framelimiter_reference.is_none()
        {
            self.framelimiter_reference = Some(std::time::Instant::now());
            self.framelimiter_count = 0;
            self.framelimiter_delta = self.target_frametime;
        }
    }
    pub (crate) fn cycle_framerate_limiter(&mut self)
    {
        let tolerated_desync =
        if self.framelimiter_check_desync
        {
            let tolerated_desync_min = 0.011;
            if 0.75*self.target_frametime > tolerated_desync_min
            {
                0.75*self.target_frametime
            }
            else
            {
                tolerated_desync_min
            }
        }
        else
        {
            0.0
        };
        self.framelimiter_count += 1;
        let time_target = self.framelimiter_reference.unwrap() + duration_from_secs(self.framelimiter_count as f64 * self.target_frametime);
        let time_actual = std::time::Instant::now();
        let to_wait =
        if time_actual > time_target
        {
            -duration_to_secs(&(time_actual - time_target))
        }
        else
        {
            duration_to_secs(&(time_target - time_actual))
        };
        if to_wait > 0.0
        {
            println!("fit");
            std::thread::sleep(duration_from_secs(to_wait));
            self.framelimiter_delta = self.target_frametime;
            self.framelimiter_check_desync = true;
        }
        else if -to_wait > tolerated_desync
        {
            self.framelimiter_reference = Some(time_actual);
            self.framelimiter_count = 0;
            self.framelimiter_delta = self.target_frametime + -to_wait;
            self.framelimiter_check_desync = false;
        }
        self.recent_deltas.push(self.framelimiter_delta);
        while self.recent_deltas.iter().sum::<f64>() > 1.0
        {
            let next = self.recent_deltas[1..self.recent_deltas.len()].iter().map(|x| *x).sum::<f64>();
            if next < 1.0
            {
                break;
            }
            self.recent_deltas.remove(0);
        }
    }
}
