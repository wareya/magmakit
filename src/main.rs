use std::io::{Read, BufReader};
use std::fs::File;

mod engine;
use self::engine::*;

mod input;

fn read_string(file : &mut BufReader<File>, fname : &str) -> Result<String, String>
{
    let mut string = String::new();
    file.read_to_string(&mut string).or_else(|_| Err(format!("error: failed to load file `{}` into string", fname)))?;
    Ok(string)
}

fn open_file(root : &String, prefix : &String, fname : &str) -> Result<BufReader<File>, String>
{
    if let Ok(f) = File::open(format!("{}/{}/{}", root, prefix, fname))
    {
        Ok(BufReader::new(f))
    }
    // FIXME add flag to disable
    else if let Ok(f) = File::open(format!("{}/{}", prefix, fname))
    {
        Ok(BufReader::new(f))
    }
    else
    {
        return Err(format!("error: failed to open file `{}` (`{}`)", fname, format!("{}/{}", root, fname)));
    }
}

fn load_string(root : &String, prefix : &String, fname : &str) -> Result<String, String>
{
    let mut file = open_file(root, prefix, fname)?;
    return read_string(&mut file, fname);
}

use engine::bindings::*;

struct Texts {
    list: Vec<(String, f32, f32)>
}

impl Texts {
    fn new() -> Texts
    {
        Texts{list : Vec::new()}
    }
    fn add(&mut self, text : &str, x : f32, y : f32)
    {
        self.list.push((text.to_string(), x, y))
    }
}

impl Engine {
    fn init(&mut self) -> Result<(), String>
    {
        let mut texts = Texts::new();
        texts.add("FUCK", 0.0, 0.0);
        self.global.insert(texts);
        Ok(())
    }
    fn logic(&mut self) -> Result<(), String>
    {
        let mut texts = self.global.get_mut::<Texts>();
        //texts.add("GOD DAMN", 10.0, 24.0); // works fine but creates infinite texts
        Ok(())
    }
    fn draw(&mut self) -> Result<(), String>
    {
        for text in &self.global.get::<Texts>().list
        {
            self.renderer.draw_text(&text.0, text.1, text.2);
        }
        Ok(())
    }
}

fn launch(prefix : &str) -> Result<(), String>
{
    use glium::glutin;
    
    let mut program_path = std::env::current_exe().unwrap();
    program_path.pop();
    let program_path = program_path.to_str().unwrap().to_string();
    
    println!("running from path `{}` with prefix `{}`", program_path, prefix);
    let prefix = prefix.to_string();
    
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    
    use std::rc::Rc;
    use std::cell::RefCell;
    let engine = Rc::new(RefCell::new(Engine::load(display, program_path.clone(), prefix.clone())));
    if let Ok(mut engine) = engine.try_borrow_mut()
    {
        engine.init()?;
    }
    
    let mut closed = false;
    
    while !closed
    {
        if let Ok(mut engine) = engine.try_borrow_mut()
        {
            engine.check_init_framerate_limiter();
            
            engine.input.cycle();
            
            events_loop.poll_events(|event|
            {
                use glium::glutin::{Event::WindowEvent, WindowEvent::*, Event::DeviceEvent, DeviceEvent::MouseMotion};
                match event
                {
                    WindowEvent{event, ..} => match event
                    {
                        CloseRequested => closed = true,
                        KeyboardInput{input, ..} => engine.input.keyevent(input),
                        MouseInput{state, button, ..} => engine.input.mousebuttonevent(state, button),
                        //CursorMoved{position, ..} => engine.input.mouse_pos = position.into(),
                        MouseWheel{delta, ..} => engine.input.scroll(delta),
                        _ => ()
                    },
                    DeviceEvent{event, ..} => match event
                    {
                        MouseMotion{delta, ..} =>
                        {
                            engine.input.mouse_delta.0 += delta.0;
                            engine.input.mouse_delta.1 += delta.1;
                        }
                        _ => ()
                    }
                    _ => (),
                }
            });
            engine.unsafe_check_global_cursor_position();
            
            engine.logic()?;
            
            engine.renderer.render_begin();
            
            engine.draw()?;
        
            engine.renderer.render_finish();
            engine.cycle_framerate_limiter();
        }
        else
        {
            panic!("error: failed to lock engine in mainloop");
        }
    }
    Ok(())
}

fn main() -> Result<(), String>
{
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1
    {
        launch(&args[1])
    }
    else
    {
        launch("data")
    }
}