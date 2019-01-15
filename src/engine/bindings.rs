use std::collections::VecDeque;

use std::rc::Rc;
use std::cell::RefCell;

use gammakit::Interpreter;
use gammakit::Value;
use gammakit::Custom as CustomStorage;

pub (crate) type EngineBinding = Fn(&mut Engine, VecDeque<Value>) -> Result<Value, String>;

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
            Some(Value::$type(val)) => Ok(val),
            _ => Err(format!("error: given value had wrong type in binding"))
        }
    }
} }

impl Engine {
    fn binding_sprite_load(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
    fn binding_sprite_load_with_subimages(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
    fn binding_draw_sprite(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
        
        Ok(Value::Number(0.0 as f64))
    }
    fn binding_draw_sprite_index(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
        
        Ok(Value::Number(0.0 as f64))
    }
    fn binding_key_down(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to key_down()".to_string());
        }
        let name = pop_front!(args, Text)?;
        let down = self.input_handler.keys_down.get(&name).cloned().unwrap_or(false);
        Ok(Value::Number(down as u32 as f64))
    }
    fn binding_key_pressed(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
    fn binding_key_released(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
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
    
    fn binding_sqrt(&mut self, mut args : VecDeque<Value>) -> Result<Value, String>
    {
        if args.len() != 1
        {
            return Err("error: expected exactly 1 arguments to sqrt()".to_string());
        }
        let val = pop_front!(args, Number)?;
        Ok(Value::Number(val.sqrt()))
    }
    // It's okay if you have no idea what this is doing, just pretend that RefCell is a mutex and Rc is a smart pointer.
    fn insert_binding(interpreter : &mut Interpreter, engine : &Rc<RefCell<Engine>>, name : &'static str, func : &'static EngineBinding)
    {
        let engine_ref = Rc::clone(&engine);
        interpreter.insert_simple_binding(name.to_string(), Rc::new(RefCell::new(move |args : VecDeque<Value>| -> Result<Value, String>
        {
            let mut engine = engine_ref.try_borrow_mut().or_else(|_| Err(format!("error: failed to lock engine in {}()", name)))?;
            
            func(&mut *engine, args)
        })));
    }
    
    pub (crate) fn insert_bindings(interpreter : &mut Interpreter, engine : &Rc<RefCell<Engine>>)
    {
        Engine::insert_binding(interpreter, engine, "sprite_load", &Engine::binding_sprite_load);
        Engine::insert_binding(interpreter, engine, "sprite_load_with_subimages", &Engine::binding_sprite_load_with_subimages);
        Engine::insert_binding(interpreter, engine, "draw_sprite", &Engine::binding_draw_sprite);
        Engine::insert_binding(interpreter, engine, "draw_sprite_index", &Engine::binding_draw_sprite_index);
        Engine::insert_binding(interpreter, engine, "key_down", &Engine::binding_key_down);
        Engine::insert_binding(interpreter, engine, "key_pressed", &Engine::binding_key_pressed);
        Engine::insert_binding(interpreter, engine, "key_released", &Engine::binding_key_released);
        Engine::insert_binding(interpreter, engine, "sqrt", &Engine::binding_sqrt);
    }
}
