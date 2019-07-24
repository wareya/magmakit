use std::rc::Rc;
use std::cell::RefCell;

use gammakit::Interpreter;
use gammakit::Value;
use gammakit::Custom as CustomStorage;

pub (crate) type EngineBinding = Fn(&mut Engine, Vec<Value>) -> Result<Value, String>;

use super::*;

fn build_custom(discrim : u64, storage : u64) -> Value
{
    Value::Custom(CustomStorage{discrim, storage})
}
fn match_custom(val : CustomStorage, want_discrim : u64) -> Result<u64, String>
{
    if val.discrim == want_discrim
    {
        return Ok(val.storage);
    }
    Err(format!("error: expected Custom value with discriminator {}, got {}", want_discrim, val.discrim))
}

macro_rules! pop_front { ( $list:expr, $type:ident )  =>
{
    if $list.is_empty()
    {
        Err(format!("error: failed to get value in binding"))
    }
    else
    {
        match $list.remove(0)
        {
            Value::$type(val) => Ok(val),
            _ => Err(format!("error: given value had wrong type in binding"))
        }
    }
} }

fn default_return() -> Result<Value, String>
{
    Ok(Value::Number(0.0 as f64))
}

impl Engine {
    fn binding_program_load(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 2
        {
            return Err("error: expected exactly 2 arguments to program_load()".to_string());
        }
        let filename_vertex = pop_front!(args, Text)?;
        let filename_fragment = pop_front!(args, Text)?;
        
        Ok(build_custom(1, self.load_program(&filename_vertex, &filename_fragment)?))
    }
    fn binding_program_set(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 argument to program_set()".to_string());
        }
        let program_index_wrapped = pop_front!(args, Custom)?;
        let program_index = match_custom(program_index_wrapped, 1)?;
        
        self.set_program(program_index)?;
        default_return()
    }
    fn binding_program_reset(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to program_set()".to_string());
        }
        self.reset_program();
        default_return()
    }
    fn binding_draw_text(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 3
        {
            return Err("error: expected exactly 3 arguments to draw_text()".to_string());
        }
        let text = pop_front!(args, Text)?;
        let x = pop_front!(args, Number)? as f32;
        let y = pop_front!(args, Number)? as f32;
        self.draw_text(&text, x, y, 999999999.0, 999999999.0, 24.0, [1.0, 1.0, 1.0, 1.0]);
        
        default_return()
    }
    fn binding_draw_text_ext(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 6
        {
            return Err("error: expected exactly 6 arguments to draw_text_ext()".to_string());
        }
        let text = pop_front!(args, Text)?;
        let x = pop_front!(args, Number)? as f32;
        let y = pop_front!(args, Number)? as f32;
        let w = pop_front!(args, Number)? as f32;
        let h = pop_front!(args, Number)? as f32;
        let size = pop_front!(args, Number)? as f32;
        self.draw_text(&text, x, y, w, h, size, [1.0, 1.0, 1.0, 1.0]);
        
        default_return()
    }
    fn binding_sprite_load(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 3
        {
            return Err("error: expected exactly 3 arguments to sprite_load()".to_string());
        }
        let filename = pop_front!(args, Text)?;
        let xoffset = pop_front!(args, Number)?;
        let yoffset = pop_front!(args, Number)?;
        
        Ok(build_custom(0, self.load_sprite(&filename, (xoffset, yoffset))))
    }
    fn binding_sprite_load_with_subimages(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 2
        {
            return Err("error: expected exactly 2 arguments to sprite_load_with_subimages()".to_string());
        }
        let filename = pop_front!(args, Text)?;
        let mut subimages = pop_front!(args, Array)?;
        
        let mut subimages_vec = Vec::new();
        while !subimages.is_empty()
        {
            let mut subimage = pop_front!(subimages, Array)?;
            macro_rules! pop { () => { pop_front!(subimage, Number)? } }
            subimages_vec.push(SpriteImage::extended((pop!(), pop!()), (pop!(), pop!()), (pop!(), pop!())));
        }
        if subimages_vec.is_empty()
        {
            return Err("error: sprite_load_with_subimages must be given at least one subimage".to_string());
        }
        
        Ok(build_custom(0, self.load_sprite_with_subimages(&filename, subimages_vec)))
    }
    fn binding_draw_sprite(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 3
        {
            return Err("error: expected exactly 3 arguments to draw_sprite()".to_string());
        }
        let sprite_index_wrapped = pop_front!(args, Custom)?;
        let sprite_index = match_custom(sprite_index_wrapped, 0)?;
        let x = pop_front!(args, Number)? as f32;
        let y = pop_front!(args, Number)? as f32;
        self.draw_sprite(sprite_index, 0, x, y);
        
        default_return()
    }
    fn binding_draw_sprite_scaled(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 5
        {
            return Err("error: expected exactly 5 arguments to draw_sprite_scaled()".to_string());
        }
        let sprite_index_wrapped = pop_front!(args, Custom)?;
        let sprite_index = match_custom(sprite_index_wrapped, 0)?;
        let x = pop_front!(args, Number)? as f32;
        let y = pop_front!(args, Number)? as f32;
        let xscale = pop_front!(args, Number)? as f32;
        let yscale = pop_front!(args, Number)? as f32;
        self.draw_sprite_scaled(sprite_index, 0, x, y, xscale, yscale);
        
        default_return()
    }
    fn binding_draw_sprite_index(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 4
        {
            return Err("error: expected exactly 4 arguments to draw_sprite_index()".to_string());
        }
        let sprite_index_wrapped = pop_front!(args, Custom)?;
        let sprite_index = match_custom(sprite_index_wrapped, 0)?;
        
        let image_index = pop_front!(args, Number)?.floor() as u64;
        let x = pop_front!(args, Number)? as f32;
        let y = pop_front!(args, Number)? as f32;
        
        self.draw_sprite(sprite_index, image_index, x, y);
        
        default_return()
    }
    fn binding_key_down(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to key_down()".to_string());
        }
        let name = pop_front!(args, Text)?;
        let down = self.input_handler.keys_down.get(&name).cloned().unwrap_or(false);
        Ok(Value::Number(down as u32 as f64))
    }
    fn binding_key_pressed(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to key_pressed()".to_string());
        }
        let name = pop_front!(args, Text)?;
        let down = self.input_handler.keys_down.get(&name).cloned().unwrap_or(false);
        let down_previous = self.input_handler.keys_down_previous.get(&name).cloned().unwrap_or(false);
        Ok(Value::Number((down && !down_previous) as u32 as f64))
    }
    fn binding_key_released(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to key_released()".to_string());
        }
        let name = pop_front!(args, Text)?;
        let down = self.input_handler.keys_down.get(&name).cloned().unwrap_or(false);
        let down_previous = self.input_handler.keys_down_previous.get(&name).cloned().unwrap_or(false);
        Ok(Value::Number((!down && down_previous) as u32 as f64))
    }
    
    fn binding_mouse_cursor_disable(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to mouse_cursor_disable()".to_string());
        }
        self.display.gl_window().window().hide_cursor(true);
        
        default_return()
    }
    fn binding_mouse_cursor_enable(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to mouse_cursor_enable()".to_string());
        }
        self.display.gl_window().window().hide_cursor(false);
        
        default_return()
    }
    
    fn binding_mouse_position(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to mouse_position()".to_string());
        }
        let (x, y) = self.input_handler.mouse_pos;
        Ok(Value::Array(vec!(Value::Number(x), Value::Number(y))))
    }
    fn binding_mouse_position_x(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to mouse_position_x()".to_string());
        }
        Ok(Value::Number(self.input_handler.mouse_pos.0))
    }
    fn binding_mouse_position_y(&mut self, args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to mouse_position_y()".to_string());
        }
        Ok(Value::Number(self.input_handler.mouse_pos.1))
    }
    fn binding_mouse_button_down(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to mouse_button_down()".to_string());
        }
        let button = pop_front!(args, Number)?.round();
        if button >= 5.0 || button < 0.0
        {
            return default_return();
        }
        let down = self.input_handler.mouse_buttons[button as usize];
        Ok(Value::Number(down as u32 as f64))
    }
    fn binding_mouse_button_pressed(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to mouse_button_pressed()".to_string());
        }
        let button = pop_front!(args, Number)?.round();
        if button >= 5.0 || button < 0.0
        {
            return default_return();
        }
        let down = self.input_handler.mouse_buttons[button as usize];
        let down_previous = self.input_handler.mouse_buttons_previous[button as usize];
        Ok(Value::Number((down && !down_previous) as u32 as f64))
    }
    fn binding_mouse_button_released(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to mouse_button_released()".to_string());
        }
        let button = pop_front!(args, Number)?.round();
        if button >= 5.0 || button < 0.0
        {
            return default_return();
        }
        let down = self.input_handler.mouse_buttons[button as usize];
        let down_previous = self.input_handler.mouse_buttons_previous[button as usize];
        Ok(Value::Number((!down && down_previous) as u32 as f64))
    }
    
    fn binding_sqrt(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to sqrt()".to_string());
        }
        let val = pop_front!(args, Number)?;
        Ok(Value::Number(val.sqrt()))
    }
    
    fn binding_set_framerate(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to set_framerate()".to_string());
        }
        let mut val = pop_front!(args, Number)?;
        if val < 1.0
        {
            val = 1.0;
        }
        self.target_frametime = 1.0/val;
        self.framelimiter_reference = self.framelimiter_reset_reference_time;
        self.framelimiter_count = 0;
        default_return()
    }
    fn binding_set_frametime(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to set_frametime()".to_string());
        }
        let mut val = pop_front!(args, Number)?;
        if val < 1.0
        {
            val = 1.0;
        }
        self.target_frametime = val;
        default_return()
    }
    fn binding_get_target_framerate(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_target_framerate()".to_string());
        }
        Ok(Value::Number(1.0/self.target_frametime))
    }
    fn binding_get_target_frametime(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_target_frametime()".to_string());
        }
        Ok(Value::Number(self.target_frametime))
    }
    fn binding_get_immediate_framerate(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_immediate_framerate()".to_string());
        }
        Ok(Value::Number(1.0/self.framelimiter_delta))
    }
    fn binding_get_smooth_framerate(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_smooth_framerate()".to_string());
        }
        Ok(Value::Number(1.0/(self.recent_deltas.iter().sum::<f64>() / self.recent_deltas.len() as f64)))
    }
    fn binding_get_perceptual_framerate(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_perceptual_framerate()".to_string());
        }
        let total_delta = self.recent_deltas.iter().sum::<f64>() as f64;
        let broken_deltas = self.recent_deltas.iter().map(|x| x*x/total_delta/total_delta).collect::<Vec<_>>();
        let avg_broken_delta = total_delta * broken_deltas.iter().sum::<f64>();
        Ok(Value::Number(1.0/avg_broken_delta))
    }
    fn binding_get_frame_delta_secs(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_frame_delta_secs()".to_string());
        }
        Ok(Value::Number(self.framelimiter_delta))
    }
    fn binding_get_frame_delta_msecs(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 0
        {
            return Err("error: expected exactly 0 arguments to get_frame_delta_msecs()".to_string());
        }
        Ok(Value::Number(self.framelimiter_delta*1000.0))
    }
    fn binding_get_frame_delta_frames(&mut self, mut args : Vec<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to get_frame_delta_frames()".to_string());
        }
        let mut val = pop_front!(args, Number)?;
        Ok(Value::Number(val/self.framelimiter_delta))
    }
    // It's okay if you have no idea what this is doing, just pretend that RefCell is a mutex and Rc is a smart pointer.
    fn insert_binding(interpreter : &mut Interpreter, engine : &Rc<RefCell<Engine>>, name : &'static str, func : &'static EngineBinding)
    {
        let engine_ref = Rc::clone(&engine);
        interpreter.insert_simple_binding(name.to_string(), Rc::new(RefCell::new(move |args : Vec<Value>| -> Result<Value, String>
        {
            let mut engine = engine_ref.try_borrow_mut().or_else(|_| Err(format!("error: failed to lock engine in {}()", name)))?;
            
            func(&mut *engine, args)
        })));
    }
    
    pub (crate) fn insert_bindings(interpreter : &mut Interpreter, engine : &Rc<RefCell<Engine>>)
    {
        Engine::insert_binding(interpreter, engine, "sqrt", &Engine::binding_sqrt);
        
        Engine::insert_binding(interpreter, engine, "program_load", &Engine::binding_program_load);
        Engine::insert_binding(interpreter, engine, "program_set", &Engine::binding_program_set);
        Engine::insert_binding(interpreter, engine, "program_reset", &Engine::binding_program_reset);
        
        Engine::insert_binding(interpreter, engine, "sprite_load", &Engine::binding_sprite_load);
        Engine::insert_binding(interpreter, engine, "sprite_load_with_subimages", &Engine::binding_sprite_load_with_subimages);
        
        Engine::insert_binding(interpreter, engine, "draw_text", &Engine::binding_draw_text);
        Engine::insert_binding(interpreter, engine, "draw_text_ext", &Engine::binding_draw_text_ext);
        Engine::insert_binding(interpreter, engine, "draw_sprite", &Engine::binding_draw_sprite);
        Engine::insert_binding(interpreter, engine, "draw_sprite_scaled", &Engine::binding_draw_sprite_scaled);
        Engine::insert_binding(interpreter, engine, "draw_sprite_index", &Engine::binding_draw_sprite_index);
        
        Engine::insert_binding(interpreter, engine, "key_down", &Engine::binding_key_down);
        Engine::insert_binding(interpreter, engine, "key_pressed", &Engine::binding_key_pressed);
        Engine::insert_binding(interpreter, engine, "key_released", &Engine::binding_key_released);
        
        Engine::insert_binding(interpreter, engine, "mouse_position", &Engine::binding_mouse_position);
        Engine::insert_binding(interpreter, engine, "mouse_position_x", &Engine::binding_mouse_position_x);
        Engine::insert_binding(interpreter, engine, "mouse_position_y", &Engine::binding_mouse_position_y);
        Engine::insert_binding(interpreter, engine, "mouse_button_down", &Engine::binding_mouse_button_down);
        Engine::insert_binding(interpreter, engine, "mouse_button_pressed", &Engine::binding_mouse_button_pressed);
        Engine::insert_binding(interpreter, engine, "mouse_button_released", &Engine::binding_mouse_button_released);
        
        Engine::insert_binding(interpreter, engine, "mouse_cursor_enable", &Engine::binding_mouse_cursor_enable);
        Engine::insert_binding(interpreter, engine, "mouse_cursor_disable", &Engine::binding_mouse_cursor_disable);
        
        Engine::insert_binding(interpreter, engine, "set_framerate", &Engine::binding_set_framerate);
        Engine::insert_binding(interpreter, engine, "set_frametime", &Engine::binding_set_frametime);
        Engine::insert_binding(interpreter, engine, "get_target_framerate", &Engine::binding_get_target_framerate);
        Engine::insert_binding(interpreter, engine, "get_target_frametime", &Engine::binding_get_target_frametime);
        Engine::insert_binding(interpreter, engine, "get_immediate_framerate", &Engine::binding_get_immediate_framerate);
        Engine::insert_binding(interpreter, engine, "get_smooth_framerate", &Engine::binding_get_smooth_framerate);
        Engine::insert_binding(interpreter, engine, "get_perceptual_framerate", &Engine::binding_get_perceptual_framerate);
        Engine::insert_binding(interpreter, engine, "get_frame_delta_secs", &Engine::binding_get_frame_delta_secs);
        Engine::insert_binding(interpreter, engine, "get_frame_delta_msecs", &Engine::binding_get_frame_delta_msecs);
        Engine::insert_binding(interpreter, engine, "get_frame_delta_frames", &Engine::binding_get_frame_delta_frames);
    }
}
