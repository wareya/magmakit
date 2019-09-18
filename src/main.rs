use std::io::{Read, BufReader};
use std::fs::File;

mod engine;
use self::engine::*;

mod input;
mod collision;

fn read_string(file : &mut BufReader<File>, fname : &str) -> Result<String, String>
{
    let mut string = String::new();
    file.read_to_string(&mut string).or_else(|_| Err(format!("error: failed to load file `{}` into string", fname)))?;
    Ok(string)
}

fn open_file(root : &String, fname : &str) -> Result<BufReader<File>, String>
{
    if let Ok(f) = File::open(format!("{}/{}", root, fname))
    {
        Ok(BufReader::new(f))
    }
    // FIXME add flag to disable
    else if let Ok(f) = File::open(fname)
    {
        Ok(BufReader::new(f))
    }
    else
    {
        return Err(format!("error: failed to open file `{}`", fname));
    }
}

fn load_string(root : &String, fname : &str) -> Result<String, String>
{
    let mut file = open_file(root, fname)?;
    return read_string(&mut file, fname);
}

fn main() -> Result<(), String>
{
    use glium::glutin;
    
    let mut program_path = std::env::current_exe().unwrap();
    program_path.pop();
    let program_path = program_path.to_str().unwrap().to_string();
    
    println!("running from path `{}`", program_path);
    
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    
    use std::rc::Rc;
    use std::cell::RefCell;
    let engine = Rc::new(RefCell::new(Engine::load(display, program_path.clone())));
    
    use gammakit::Interpreter;
    let mut interpreter = Interpreter::new(gammakit::Parser::new_from_default().unwrap());
    interpreter.insert_default_bindings();
    Engine::insert_bindings(&mut interpreter, &engine);
    
    let gmc_init = interpreter.restart_into_string(&load_string(&program_path, "data/gmc/init.gmc").unwrap()).unwrap();
    let gmc_step = interpreter.restart_into_string(&load_string(&program_path, "data/gmc/step.gmc").unwrap()).unwrap();
    let gmc_draw = interpreter.restart_into_string(&load_string(&program_path, "data/gmc/draw.gmc").unwrap()).unwrap();
    interpreter.restart(&gmc_init);
    
    interpreter.step_until_error_or_exit()?;
    
    let mut closed = false;
    
    while !closed
    {
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
            engine.check_init_framerate_limiter();
            
            engine.input_handler.cycle();
            
            events_loop.poll_events(|event|
            {
                use glium::glutin::{Event::WindowEvent, WindowEvent::*, Event::DeviceEvent, DeviceEvent::MouseMotion};
                match event
                {
                    WindowEvent{event, ..} => match event
                    {
                        CloseRequested => closed = true,
                        KeyboardInput{input, ..} => engine.input_handler.keyevent(input),
                        MouseInput{state, button, ..} => engine.input_handler.mousebuttonevent(state, button),
                        //CursorMoved{position, ..} => engine.input_handler.mouse_pos = position.into(),
                        MouseWheel{delta, ..} => engine.input_handler.scroll(delta),
                        _ => ()
                    },
                    DeviceEvent{event, ..} => match event
                    {
                        MouseMotion{delta, ..} =>
                        {
                            engine.input_handler.mouse_delta.0 += delta.0;
                            engine.input_handler.mouse_delta.1 += delta.1;
                        }
                        _ => ()
                    }
                    _ => (),
                }
            });
            engine.unsafe_check_global_cursor_position();
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
        
        interpreter.restart(&gmc_step);
        interpreter.step_until_error_or_exit()?;
        
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
            engine.render_begin();
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
        
        interpreter.restart(&gmc_draw);
        interpreter.step_until_error_or_exit()?;
        
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
            engine.render_finish();
            engine.cycle_framerate_limiter();
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
    }
    Ok(())
}