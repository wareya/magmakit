use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::input::InputHandler;
use crate::{open_file, load_string};

pub mod render;
pub mod bindings;

use self::render::*;

use std::any::{Any, TypeId};

pub struct GlobalStore {
    store : HashMap<TypeId, Box<dyn Any>>
}

impl GlobalStore {
    pub fn new() -> GlobalStore
    {
        GlobalStore{store : HashMap::new()}
    }
    pub fn insert<T: Any>(&mut self, item : T)
    {
        let typeid = TypeId::of::<T>();
        let mut entry = self.store.insert(typeid, Box::new(item));
    }
    pub fn get<'a, T: Any>(&'a self) -> &'a T
    {
        let typeid = TypeId::of::<T>();
        match self.store.get(&typeid).unwrap().downcast_ref::<T>()
        {
            Some(item) =>
            {
                item
            }
            _ => panic!("dynamic typing error in GlobalStore")
        }
    }
    pub fn get_mut<'a, T: Any>(&'a mut self) -> &'a mut T
    {
        let typeid = TypeId::of::<T>();
        match self.store.get_mut(&typeid).unwrap().downcast_mut::<T>()
        {
            Some(item) =>
            {
                item
            }
            _ => panic!("dynamic typing error in GlobalStore")
        }
    }
}

pub struct Engine {
    program_path: String,
    prefix: String,
    
    pub renderer: Renderer,
    
    pub (crate) input: InputHandler,
    
    target_frametime: f64, // seconds
    framelimiter_reset_reference_time: Option<std::time::Instant>,
    framelimiter_reference: Option<std::time::Instant>,
    framelimiter_count: u64,
    framelimiter_delta: f64,
    framelimiter_check_desync: bool,
    recent_deltas: Vec<f64>,
    
    pub global: GlobalStore,
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
    pub (crate) fn load(display : glium::Display, program_path : String, prefix : String) -> Engine
    {
        Engine {
            renderer : Renderer::new(display, program_path.clone(), prefix.clone()),
            
            program_path,
            prefix,
            
            input : InputHandler::new(),
            
            target_frametime : 0.008,
            framelimiter_reset_reference_time : None,
            framelimiter_reference : None,
            framelimiter_count: 0,
            framelimiter_delta: 0.0,
            framelimiter_check_desync: false,
            recent_deltas: Vec::new(),
            
            global: GlobalStore::new(),
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
                let factor = self.renderer.display.gl_window().window().get_hidpi_factor();
                let offset = self.renderer.display.gl_window().window().get_inner_position().unwrap();
                let new_x = factor*cursor_x as f64 - offset.x;
                let new_y = factor*cursor_y as f64 - offset.y;
                self.input.mouse_pos = (new_x, new_y);
            }
        }
    }
    pub (crate) fn check_init_framerate_limiter(&mut self)
    {
        if self.framelimiter_reference.is_none()
        {
            let now = std::time::Instant::now();
            self.framelimiter_reset_reference_time = Some(now);
            self.framelimiter_reference = Some(now);
            self.framelimiter_count = 0;
            self.framelimiter_delta = self.target_frametime;
        }
    }
    pub (crate) fn cycle_framerate_limiter(&mut self)
    {
        let tolerated_desync = // amount that the engine is allowed to miss the desired wakeup time by without resetting the framerate limiter
        if self.framelimiter_check_desync
        {
            let tolerated_desync_min = 0.011; // ms
            if 0.66*self.target_frametime > tolerated_desync_min // or 0.66 of the desired frame time (gives 11ms for 60fps)
            {
                0.66*self.target_frametime
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
            std::thread::sleep(duration_from_secs(to_wait));
            self.framelimiter_reset_reference_time = Some(time_target);
            self.framelimiter_delta = self.target_frametime;
            self.framelimiter_check_desync = true;
        }
        else if -to_wait > tolerated_desync
        {
            self.framelimiter_reset_reference_time = Some(time_actual);
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
