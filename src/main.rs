use std::collections::{VecDeque, HashMap};
use glium::{implement_vertex, uniform};
use std::io::{Read, BufReader};
use std::fs::File;

fn m4mult(a : &[[f32; 4]; 4], b : &[[f32; 4]; 4]) -> [[f32; 4]; 4]
{
    let mut output = [[0f32; 4]; 4];
    for y in 0..4
    {
        for x in 0..4
        {
            output[x][y] += a[0][y] * b[x][0];
            output[x][y] += a[1][y] * b[x][1];
            output[x][y] += a[2][y] * b[x][2];
            output[x][y] += a[3][y] * b[x][3];
        }
    }
    output
}

macro_rules! err_none_or_panic { ( $x:expr )  =>
{
    match $x
    {
        Err(Some(err)) => panic!("{}", err),
        _ => ()
    }
} }

fn read_bytes(file : &mut BufReader<File>, fname : &str) -> Result<Vec<u8>, String>
{
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).or_else(|_| Err(format!("error: failed to load file `{}` into a binary buffer", fname)))?;
    Ok(bytes)
}
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

fn load_bytes(root : &String, fname : &str) -> Result<Vec<u8>, String>
{
    let mut file = open_file(root, fname)?;
    return read_bytes(&mut file, fname);
}
fn load_string(root : &String, fname : &str) -> Result<String, String>
{
    let mut file = open_file(root, fname)?;
    return read_string(&mut file, fname);
}



fn main()
{
    use glium::{glutin, Surface};
    use glium::texture::SrgbTexture2d;
    use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
    
    let mut program_path = std::env::current_exe().unwrap();
    program_path.pop();
    let program_path = program_path.to_str().unwrap().to_string();
    
    println!("running from path `{}`", program_path);
    
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0));
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2]
    }
    
    implement_vertex!(Vertex, position);
    
    struct SpriteImage {
        origin: (f64, f64),
        topleft: (f64, f64),
        bottomright: (f64, f64),
    }
    
    impl SpriteImage {
        fn basic(origin: (f64, f64), tex : &SrgbTexture2d) -> SpriteImage
        {
            SpriteImage{origin, topleft: (0.0, 0.0), bottomright: (tex.width() as f64, tex.height() as f64)}
        }
        fn extended(origin: (f64, f64), topleft: (f64, f64), bottomright : (f64, f64)) -> SpriteImage
        {
            SpriteImage{origin, topleft, bottomright}
        }
    }
    
    struct SpriteSheet {
        images: Vec<SpriteImage>,
        texture: SrgbTexture2d,
    }
    
    struct DrawEvent {
        matrix: [[f32; 4]; 4],
        spritesheet: u64,
        imageindex: u64
    }
    
    struct InputHandler {
        keys_down_previous: HashMap<String, bool>,
        keys_down: HashMap<String, bool>,
        mouse_pos: (f64, f64),
        mouse_delta: (f64, f64),
        mouse_buttons: [bool; 5],
    }
    
    impl InputHandler {
        fn new() -> InputHandler
        {
            InputHandler{keys_down : HashMap::new(), keys_down_previous : HashMap::new(), mouse_pos: (0.0, 0.0), mouse_delta: (0.0, 0.0), mouse_buttons: [false, false, false, false, false]}
        }
        fn keyevent(&mut self, event : glutin::KeyboardInput)
        {
            if let Some(key) = event.virtual_keycode
            {
                use glium::glutin::VirtualKeyCode::*;
                let keystr = match key
                {
                    // esc
                    Escape => "Esc",
                    
                    // number row
                    Key0 => "0",
                    Key1 => "1",
                    Key2 => "2",
                    Key3 => "3",
                    Key4 => "4",
                    Key5 => "5",
                    Key6 => "6",
                    Key7 => "7",
                    Key8 => "8",
                    Key9 => "9",
                    
                    // letters
                    A => "A",
                    B => "B",
                    C => "C",
                    D => "D",
                    E => "E",
                    F => "F",
                    G => "G",
                    H => "H",
                    I => "I",
                    J => "J",
                    K => "K",
                    L => "L",
                    M => "M",
                    N => "N",
                    O => "O",
                    P => "P",
                    Q => "Q",
                    R => "R",
                    S => "S",
                    T => "T",
                    U => "U",
                    V => "V",
                    W => "W",
                    X => "X",
                    Y => "Y",
                    Z => "Z",
                    
                    // navigation block (bottom)
                    Left => "Left",
                    Up => "Up",
                    Right => "Right",
                    Down => "Down",
                    
                    // typographic control
                    Back => "Backspace",
                    Return => "Enter",
                    Space => "Space",
                    Tab => "Tab",
                    
                    // punctuation
                    Grave => "`",
                    
                    Minus => "-",
                    Equals => "=",
                    
                    LBracket => "[",
                    RBracket => "]",
                    Backslash => "\\",
                    
                    Semicolon => ";",
                    Apostrophe => "'",
                    
                    Comma => ",",
                    Period => ".",
                    Slash => "/",
                    
                    // navigation block (top)
                    Scroll => "ScollLock",
                    Pause => "Pause",
                    Insert => "Insert",
                    Delete => "Delete",
                    Home => "Home",
                    End => "End",
                    PageUp => "PageUp",
                    PageDown => "PageDown",
                    
                    // modifiers
                    LAlt => "LAlt",
                    LShift => "LShift",
                    LControl => "LControl",
                    RAlt => "RAlt",
                    RShift => "RShift",
                    RControl => "RControl",
                    
                    // numpad
                    Numpad0 => "Numpad0",
                    Numpad1 => "Numpad1",
                    Numpad2 => "Numpad2",
                    Numpad3 => "Numpad3",
                    Numpad4 => "Numpad4",
                    Numpad5 => "Numpad5",
                    Numpad6 => "Numpad6",
                    Numpad7 => "Numpad7",
                    Numpad8 => "Numpad8",
                    Numpad9 => "Numpad9",
                    
                    Divide => "Numpad/",
                    Multiply => "Numpad*",
                    Subtract => "Numpad-",
                    Add => "Numpad+",
                    Decimal => "Numpad.",
                    
                    // functions
                    F1 => "F1",
                    F2 => "F2",
                    F3 => "F3",
                    F4 => "F4",
                    F5 => "F5",
                    F6 => "F6",
                    F7 => "F7",
                    F8 => "F8",
                    F9 => "F9",
                    F10 => "F10",
                    F11 => "F11",
                    F12 => "F12",
                    
                    // unsupported
                    _ => ""
                };
                if keystr != ""
                {
                    self.keys_down.insert(keystr.to_string(), event.state == glutin::ElementState::Pressed);
                }
            }
        }
        fn cycle(&mut self)
        {
            self.keys_down_previous = self.keys_down.clone();
        }
    }
    
    struct Engine {
        program_path: String,
        
        input_handler: InputHandler,
        
        sprite_index_counter: u64,
        sprites: HashMap<u64, SpriteSheet>,
        draw_events: Vec<DrawEvent>,
        
        display: glium::Display,
        
        vertex_buffer: glium::VertexBuffer<Vertex>,
        indices: glium::index::NoIndices,
        glprogram: glium::Program
    }
    
    fn deg2rad(x : f64) -> f64
    {
        x * std::f64::consts::PI / 360.0
    }
    
    impl Engine {
        fn build_glprogram(display : &glium::Display, program_path : &String) -> glium::Program
        {
            let vertex_shader_src = load_string(program_path, "data/glsl/vertex.glsl").unwrap();
            let fragment_shader_src = load_string(program_path, "data/glsl/fragment.glsl").unwrap();
            let glprogram = glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
            
            glprogram
        }
        fn build_vertex_buffer(display : &glium::Display) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices)
        {
            let vertex1 = Vertex { position: [0.0, 0.0] };
            let vertex2 = Vertex { position: [0.0, 1.0] };
            let vertex3 = Vertex { position: [1.0, 0.0] };
            let vertex4 = Vertex { position: [1.0, 1.0] };
            let shape = vec![vertex1, vertex2, vertex3, vertex4];
            
            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
            
            (vertex_buffer, indices)
        }
        fn load(display : glium::Display, program_path : String) -> Engine
        {
            let glprogram = Engine::build_glprogram(&display, &program_path);
            let (vertex_buffer, indices) = Engine::build_vertex_buffer(&display);
            Engine{program_path, input_handler : InputHandler::new(), sprite_index_counter : 1, sprites : HashMap::new(), draw_events : Vec::new(), display, vertex_buffer, indices, glprogram}
        }
        
        fn load_sprite(&mut self, fname : &str, origin : (f64, f64)) -> u64
        {
            let index = self.sprite_index_counter;
            let image = image::load(open_file(&self.program_path, fname).unwrap(), image::ImageFormat::PNG).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
            let texture = SrgbTexture2d::new(&self.display, image).unwrap();
            
            self.sprites.insert(index, SpriteSheet{images: vec!(SpriteImage::basic(origin, &texture)), texture});
            
            self.sprite_index_counter += 1;
            index
        }
        
        fn load_sprite_with_subimages(&mut self, fname : &str, images : Vec<SpriteImage>) -> u64
        {
            let index = self.sprite_index_counter;
            let image = image::load(open_file(&self.program_path, fname).unwrap(), image::ImageFormat::PNG).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
            let texture = SrgbTexture2d::new(&self.display, image).unwrap();
            
            self.sprites.insert(index, SpriteSheet{images, texture});
            
            self.sprite_index_counter += 1;
            index
        }
        
        fn draw_sprite(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32)
        {
            self.draw_sprite_scaled(spriteindex, imageindex, x, y, 1.0, 1.0)
        }
        fn draw_sprite_scaled(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32, xscale : f32, yscale : f32)
        {
            self.draw_sprite_angled(spriteindex, imageindex, x, y, xscale, yscale, 0.0)
        }
        fn draw_sprite_angled(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32, xscale : f32, yscale : f32, angle : f32)
        {
            let angle_radians = deg2rad(angle as f64);
            let angle_cos = angle_radians.cos() as f32;
            let angle_sin = angle_radians.sin() as f32;
            
            let matrix_pos = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, 0.0, 1.0],
            ];
            let matrix_rotscale = [
                [angle_cos*xscale, -angle_sin*xscale, 0.0, 0.0],
                [-angle_sin*yscale, -angle_cos*yscale, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            
            let matrix = m4mult(&matrix_pos, &matrix_rotscale);
            
            self.draw_sprite_transformed(spriteindex, imageindex, matrix)
        }
        fn draw_sprite_transformed(&mut self, spriteindex : u64, imageindex : u64, matrix : [[f32; 4]; 4])
        {
            self.draw_events.push(DrawEvent{matrix, spritesheet : spriteindex, imageindex : imageindex})
        }
        
        fn render(&mut self)
        {
            let mut target = self.display.draw();
            
            target.clear_color(0.5, 0.5, 0.5, 1.0);
            
            let dims = target.get_dimensions();
            let x_dim = dims.0 as f32;
            let y_dim = dims.1 as f32;
            
            let matrix_view = [
                [2.0/x_dim, 0.0, 0.0, 0.0],
                [0.0, -2.0/y_dim, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0f32],
            ];
            
            for event in self.draw_events.drain(..)
            {
                let spritesheet = self.sprites.get(&event.spritesheet).unwrap();
                let texture = &spritesheet.texture;
                
                let tex_w = texture.width() as f32;
                let tex_h = texture.height() as f32;
                
                let image = spritesheet.images.get(event.imageindex as usize % spritesheet.images.len()).unwrap();
                let x_dim = (image.bottomright.0 - image.topleft.0) as f32;
                let y_dim = (image.bottomright.1 - image.topleft.1) as f32;
                let xorigin = (image.origin.0) as f32;
                let yorigin = (image.origin.1) as f32;
                
                let matrix_origin = [
                    [x_dim, 0.0, 0.0, 0.0],
                    [0.0, -y_dim, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-xorigin, yorigin, 0.0, 1.0],
                ];
                let matrix_command = m4mult(&event.matrix, &matrix_origin);
                
                let uniforms = uniform! {
                    matrix_view : matrix_view,
                    matrix_command : matrix_command,
                    tex_topleft : [image.topleft.0 as f32 / tex_w, image.topleft.1 as f32 / tex_h],
                    tex_bottomright : [image.bottomright.0 as f32 / tex_w, image.bottomright.1 as f32 / tex_h],
                    tex : Sampler::new(texture).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
                };
                target.draw(&self.vertex_buffer, &self.indices, &self.glprogram, &uniforms, &Default::default()).unwrap();
            }
            
            target.finish().unwrap();
        }
    }
    
    let engine = Rc::new(RefCell::new(Engine::load(display, program_path.clone())));
    
    let mut parser = gammakit::Parser::new_from_default().unwrap();
    let gmc_init = parser.give_me_bytecode(&load_string(&program_path, "data/gmc/init.gmc").unwrap()).unwrap();
    let gmc_step = parser.give_me_bytecode(&load_string(&program_path, "data/gmc/step.gmc").unwrap()).unwrap();
    let gmc_draw = parser.give_me_bytecode(&load_string(&program_path, "data/gmc/draw.gmc").unwrap()).unwrap();
    
    use gammakit::Interpreter;
    use gammakit::Value;
    use gammakit::Custom as CustomStorage;
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
        Err(format!("error: expected Custom value with discruminator {}, got {}", want_discrim, val.discrim))
    }
    use std::rc::Rc;
    use std::cell::RefCell;
    
    let mut interpreter = Interpreter::new(&gmc_init, Some(parser));
    
    interpreter.insert_default_bindings();
    
    type EngineBinding = Fn(&mut Engine, VecDeque<Value>) -> Result<Value, String>;
    
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
    };
    
    Engine::insert_binding(&mut interpreter, &engine, "sprite_load", &Engine::binding_sprite_load);
    Engine::insert_binding(&mut interpreter, &engine, "sprite_load_with_subimages", &Engine::binding_sprite_load_with_subimages);
    Engine::insert_binding(&mut interpreter, &engine, "draw_sprite", &Engine::binding_draw_sprite);
    Engine::insert_binding(&mut interpreter, &engine, "draw_sprite_index", &Engine::binding_draw_sprite_index);
    Engine::insert_binding(&mut interpreter, &engine, "key_down", &Engine::binding_key_down);
    Engine::insert_binding(&mut interpreter, &engine, "key_pressed", &Engine::binding_key_pressed);
    Engine::insert_binding(&mut interpreter, &engine, "key_released", &Engine::binding_key_released);
    
    
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
                use glium::glutin::{Event::WindowEvent, WindowEvent::*};
                match event
                {
                    WindowEvent { event, .. } => match event
                    {
                        CloseRequested => closed = true,
                        KeyboardInput{device_id : _, input : event} => engine.input_handler.keyevent(event),
                        _ => ()
                    },
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