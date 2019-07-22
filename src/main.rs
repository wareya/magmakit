use std::io::{Read, BufReader};
use std::fs::File;

macro_rules! err_none_or_panic { ( $x:expr )  =>
{
    match $x
    {
        Err(Some(err)) => panic!("{}", err),
        _ => ()
    }
} }

mod engine;
use self::engine::*;

mod input;

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

fn main()
{
    use glium::glutin;
    
    let mut program_path = std::env::current_exe().unwrap();
    program_path.pop();
    let program_path = program_path.to_str().unwrap().to_string();
    
    println!("running from path `{}`", program_path);
    
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    
    let engine = Rc::new(RefCell::new(Engine::load(display, program_path.clone())));
    
    let mut parser = gammakit::Parser::new_from_default().unwrap();
    let gmc_init = parser.give_me_bytecode(&load_string(&program_path, "data/gmc/init.gmc").unwrap()).unwrap();
    let gmc_step = parser.give_me_bytecode_share_bookkeeping(&load_string(&program_path, "data/gmc/step.gmc").unwrap(), &gmc_init).unwrap();
    let gmc_draw = parser.give_me_bytecode_share_bookkeeping(&load_string(&program_path, "data/gmc/draw.gmc").unwrap(), &gmc_init).unwrap();
    
    use std::rc::Rc;
    use std::cell::RefCell;
    
    use gammakit::Interpreter;
    let mut interpreter = Interpreter::new(&gmc_init, Some(parser));
    
    interpreter.insert_default_bindings();
    
    Engine::insert_bindings(&mut interpreter, &engine);
    
    fn step_until_end_maybe_panic(interpreter : &mut Interpreter)
    {
        while interpreter.step().is_ok() {}
        
        if let Some(err) = &interpreter.last_error
        {
            panic!("{}", err);
        }
    };
    
    step_until_end_maybe_panic(&mut interpreter);
    
    let mut closed = false;
    
    use std::{thread, time};
    
    let frametime = time::Duration::from_millis(8);
    
    while !closed
    {
        let frame_start = time::Instant::now();
        
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
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
                        CursorMoved{position, ..} => engine.input_handler.mouse_pos = position.into(),
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
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
        
        err_none_or_panic!(interpreter.restart(&gmc_step));
        step_until_end_maybe_panic(&mut interpreter);
        
        err_none_or_panic!(interpreter.restart(&gmc_draw));
        step_until_end_maybe_panic(&mut interpreter);
        
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
            engine.render();
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
        
        let elapsed = frame_start.elapsed();
        
        if let Some(remaining) = frametime.checked_sub(elapsed)
        {
            thread::sleep(remaining);
        }
    }
}