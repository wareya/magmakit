use std::collections::HashMap;
use glium::{implement_vertex, uniform};
use std::io::BufReader;
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


fn main()
{
    use glium::{glutin, Surface};
    use glium::texture::SrgbTexture2d;
    use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
    
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
    
    struct Renderer {
        sprite_index_counter: u64,
        sprites: HashMap<u64, SpriteSheet>,
        events: Vec<DrawEvent>,
        display: glium::Display,
        vertex_buffer: glium::VertexBuffer<Vertex>,
        indices: glium::index::NoIndices,
        program: glium::Program
    }
    
    fn deg2rad(x : f64) -> f64
    {
        x * std::f64::consts::PI / 360.0
    }
    
    impl Renderer {
        fn build_program(display : &glium::Display) -> glium::Program
        {
            let vertex_shader_src = include_str!("glsl/vertex.glsl");
            let fragment_shader_src = include_str!("glsl/fragment.glsl");
            let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();
            
            program
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
        fn load(display : glium::Display) -> Renderer
        {
            let program = Renderer::build_program(&display);
            let (vertex_buffer, indices) = Renderer::build_vertex_buffer(&display);
            Renderer{sprite_index_counter : 1, sprites : HashMap::new(), events : Vec::new(), display, vertex_buffer, indices, program}
        }
        
        fn load_sprite(&mut self, fname : &str, origin : (f64, f64)) -> u64
        {
            let index = self.sprite_index_counter;
            let image = image::load(BufReader::new(File::open(fname).unwrap()), image::ImageFormat::PNG).unwrap().to_rgba();
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
            let image = image::load(BufReader::new(File::open(fname).unwrap()), image::ImageFormat::PNG).unwrap().to_rgba();
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
            self.events.push(DrawEvent{matrix, spritesheet : spriteindex, imageindex : imageindex})
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
            
            for event in self.events.drain(..)
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
                target.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &Default::default()).unwrap();
            }
            
            target.finish().unwrap();
        }
    }
    
    let renderer = Rc::new(RefCell::new(Renderer::load(display)));
    
    let mut parser = gammakit::Parser::new_from_default().unwrap();
    let gmc_init = parser.give_me_bytecode(include_str!("gmc/init.gmc")).unwrap();
    let gmc_step = parser.give_me_bytecode(include_str!("gmc/step.gmc")).unwrap();
    let gmc_draw = parser.give_me_bytecode(include_str!("gmc/draw.gmc")).unwrap();
    
    use gammakit::Interpreter;
    use gammakit::Value;
    use gammakit::Custom as CustomStorage;
    fn build_custom(discrim : u64, storage : u64) -> Value
    {
        Value::Custom(CustomStorage{discrim, storage})
    }
    fn match_custom(val : Value, want_discrim : u64) -> Option<u64>
    {
        match val
        {
            Value::Custom(CustomStorage{discrim, storage}) =>
            {
                if discrim == want_discrim
                {
                    return Some(storage);
                }
            }
            _ => ()
        }
        None
    }
    use std::rc::Rc;
    use std::cell::RefCell;
    
    let mut interpreter = Interpreter::new(&gmc_init, Some(parser));
    
    interpreter.insert_default_bindings();
    
    type RendererBinding = Fn(&mut Renderer, Vec<Value>) -> Result<Value, String>;
    
    impl Renderer {
        fn binding_sprite_load(&mut self, mut args : Vec<Value>) -> Result<Value, String>
        {
            if args.len() != 3
            {
                return Err("error: expected exactly 3 arguments to sprite_load()".to_string());
            }
            let filename = match args.pop()
            {
                Some(Value::Text(filename)) => filename,
                _ => return Err("error: first argument to sprite_load() must be text (filename)".to_string())
            };
            let xoffset = match args.pop()
            {
                Some(Value::Number(xoffset)) => xoffset,
                _ => return Err("error: second argument to sprite_load() must be a number (xoffset)".to_string())
            };
            let yoffset = match args.pop()
            {
                Some(Value::Number(yoffset)) => yoffset,
                _ => return Err("error: third argument to sprite_load() must be a number (yoffset)".to_string())
            };
            
            let sprite_index = self.load_sprite(&filename, (xoffset, yoffset));
            
            Ok(build_custom(0, sprite_index))
        }
        fn binding_sprite_load_with_subimages(&mut self, mut args : Vec<Value>) -> Result<Value, String>
        {
            if args.len() != 2
            {
                return Err("error: expected exactly 2 arguments to sprite_load_with_subimages()".to_string());
            }
            let filename = match args.pop()
            {
                Some(Value::Text(filename)) => filename,
                _ => return Err("error: first argument to sprite_load_with_subimages() must be text (filename)".to_string())
            };
            let subimages = match args.pop()
            {
                Some(Value::Array(list)) => list,
                _ => return Err("error: second argument to sprite_load_with_subimages() must be a list (subimages)".to_string())
            };
            
            let mut subimages_vec = Vec::new();
            
            for subimage in subimages
            {
                if let Value::Array(mut subimage) = subimage
                {
                    macro_rules! pop { ( )  =>
                    {
                        match subimage.pop_front()
                        {
                            Some(Value::Number(val)) => val,
                            _ => return Err("error: each sub-array in array passed to sprite_load_with_subimages must consist of exactly six values that are numbers".to_string())
                        }
                    } }
                    subimages_vec.push(SpriteImage::extended((pop!(), pop!()), (pop!(), pop!()), (pop!(), pop!())));
                }
                else
                {
                    return Err("error: each sub-array in array passed to sprite_load_with_subimages must consist of exactly six values that are numbers".to_string());
                }
            }
            
            if subimages_vec.is_empty()
            {
                return Err("error: sprite_load_with_subimages must be given at least one subimage".to_string());
            }
            
            let sprite_index = self.load_sprite_with_subimages(&filename, subimages_vec);
            
            Ok(build_custom(0, sprite_index))
        }
        fn binding_draw_sprite(&mut self, mut args : Vec<Value>) -> Result<Value, String>
        {
            if args.len() != 3
            {
                return Err("error: expected exactly 3 arguments to draw_sprite()".to_string());
            }
            let sprite_index_wrapped = args.pop().ok_or_else(|| "unreachable error: couldn't find first argument to draw_sprite()".to_string())?;
            let sprite_index = match_custom(sprite_index_wrapped, 0).ok_or_else(|| "error: first argument to draw_sprite() must be a sprite id".to_string())?;
            let x = match args.pop()
            {
                Some(Value::Number(xoffset)) => xoffset as f32,
                _ => return Err("error: third argument to draw_sprite() must be a number (xoffset)".to_string())
            };
            let y = match args.pop()
            {
                Some(Value::Number(yoffset)) => yoffset as f32,
                _ => return Err("error: fourth argument to draw_sprite() must be a number (yoffset)".to_string())
            };
            self.draw_sprite(sprite_index, 0, x, y);
            
            Ok(Value::Number(0.0 as f64))
        }
        fn binding_draw_sprite_index(&mut self, mut args : Vec<Value>) -> Result<Value, String>
        {
            if args.len() != 4
            {
                return Err("error: expected exactly 4 arguments to draw_sprite_index()".to_string());
            }
            let sprite_index_wrapped = args.pop().ok_or_else(|| "unreachable error: couldn't find first argument to draw_sprite_index()".to_string())?;
            let sprite_index = match_custom(sprite_index_wrapped, 0).ok_or_else(|| "error: first argument to draw_sprite_index() must be a sprite id".to_string())?;
            let image_index = match args.pop()
            {
                Some(Value::Number(image_index)) => image_index.floor() as u64,
                _ => return Err("error: second argument to draw_sprite_index() must be a number (image index)".to_string())
            };
            let x = match args.pop()
            {
                Some(Value::Number(xoffset)) => xoffset as f32,
                _ => return Err("error: third argument to draw_sprite_index() must be a number (xoffset)".to_string())
            };
            let y = match args.pop()
            {
                Some(Value::Number(yoffset)) => yoffset as f32,
                _ => return Err("error: fourth argument to draw_sprite_index() must be a number (yoffset)".to_string())
            };
            self.draw_sprite(sprite_index, image_index, x, y);
            
            Ok(Value::Number(0.0 as f64))
        }
        // It's okay if you have no idea what this is doing, just pretend that RefCell is a mutex and Rc is a smart pointer.
        fn insert_binding(interpreter : &mut Interpreter, renderer : &Rc<RefCell<Renderer>>, name : &'static str, func : &'static RendererBinding)
        {
            let renderer_ref = Rc::clone(&renderer);
            interpreter.insert_simple_binding(name.to_string(), Rc::new(RefCell::new(move |args : Vec<Value>| -> Result<Value, String>
            {
                let mut renderer = renderer_ref.try_borrow_mut().or_else(|_| Err(format!("error: failed to lock renderer in {}()", name)))?;
                
                func(&mut *renderer, args)
            })));
        }
    };
    
    Renderer::insert_binding(&mut interpreter, &renderer, "sprite_load", &Renderer::binding_sprite_load);
    Renderer::insert_binding(&mut interpreter, &renderer, "sprite_load_with_subimages", &Renderer::binding_sprite_load_with_subimages);
    Renderer::insert_binding(&mut interpreter, &renderer, "draw_sprite", &Renderer::binding_draw_sprite);
    Renderer::insert_binding(&mut interpreter, &renderer, "draw_sprite_index", &Renderer::binding_draw_sprite_index);
    
    
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
        
        err_none_or_panic!(interpreter.restart(&gmc_step));
        step_until_end_maybe_panic(&mut interpreter);
        
        err_none_or_panic!(interpreter.restart(&gmc_draw));
        step_until_end_maybe_panic(&mut interpreter);
        
        if let Ok(mut renderer) = renderer.try_borrow_mut()
        {
            renderer.render();
            
            events_loop.poll_events(|event|
            {
                match event
                {
                    glutin::Event::WindowEvent { event, .. } => match event
                    {
                        glutin::WindowEvent::CloseRequested => closed = true,
                        _ => ()
                    },
                    _ => (),
                }
            });
        }
        else
        {
            panic!("error: failed to lock renderer in mainloop");
        }
        
        let elapsed = frame_start.elapsed();
        
        if let Some(remaining) = frametime.checked_sub(elapsed)
        {
            thread::sleep(remaining);
        }
    }
}