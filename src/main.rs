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
    
    struct Sprite {
        origin: (f64, f64),
        texture: SrgbTexture2d
    }
    
    struct DrawEvent {
        matrix: [[f32; 4]; 4],
        sprite: String
    }
    
    struct Renderer {
        sprites: HashMap<String, Sprite>,
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
        fn build_program(display : &glium::Display) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices, glium::Program)
        {
            let vertex1 = Vertex { position: [0.0, 0.0] };
            let vertex2 = Vertex { position: [0.0, 1.0] };
            let vertex3 = Vertex { position: [1.0, 0.0] };
            let vertex4 = Vertex { position: [1.0, 1.0] };
            let shape = vec![vertex1, vertex2, vertex3, vertex4];
            
            let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
            
            let vertex_shader_src = include_str!("glsl/vertex.glsl");
            let fragment_shader_src = include_str!("glsl/fragment.glsl");
            let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();
            
            (vertex_buffer, indices, program)
        }
        fn load(display : glium::Display) -> Renderer
        {
            let (vertex_buffer, indices, program) = Renderer::build_program(&display);
            Renderer{sprites : HashMap::new(), events : Vec::new(), display, vertex_buffer, indices, program}
        }
        
        fn load_sprite(&mut self, name : String, fname : &str, origin : (f64, f64))
        {
            let image = image::load(BufReader::new(File::open(fname).unwrap()), image::ImageFormat::PNG).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
            let texture = SrgbTexture2d::new(&self.display, image).unwrap();
            
            self.sprites.insert(name, Sprite{origin, texture});
        }
        
        fn draw(&mut self, spritename : &str, x : f32, y : f32)
        {
            self.draw_scaled(spritename, x, y, 1.0, 1.0)
        }
        fn draw_scaled(&mut self, spritename : &str, x : f32, y : f32, xscale : f32, yscale : f32)
        {
            self.draw_angle(spritename, x, y, xscale, yscale, 0.0)
        }
        fn draw_angle(&mut self, spritename : &str, x : f32, y : f32, xscale : f32, yscale : f32, angle : f32)
        {
            let angle_radians = deg2rad(angle as f64);
            let angle_cos = angle_radians.cos() as f32;
            let angle_sin = angle_radians.sin() as f32;
            let sprite = self.sprites.get(spritename).unwrap();
            let x_dim = sprite.texture.dimensions().0 as f32;
            let y_dim = sprite.texture.dimensions().1 as f32;
            let xorigin = sprite.origin.0 as f32;
            let yorigin = sprite.origin.1 as f32;
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
            let matrix_origin = [
                [x_dim, 0.0, 0.0, 0.0],
                [0.0, -y_dim, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-xorigin, yorigin, 0.0, 1.0],
            ];
            let mut matrix = m4mult(&matrix_pos, &matrix_rotscale);
            matrix = m4mult(&matrix, &matrix_origin);
            
            self.events.push(DrawEvent{matrix, sprite : spritename.to_string()})
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
                let texture = &self.sprites.get(&event.sprite).unwrap().texture;
                let uniforms = uniform! {
                    matrix_view: matrix_view,
                    matrix_command: event.matrix,
                    tex: Sampler::new(texture).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
                };
                target.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &Default::default()).unwrap();
            }
            
            target.finish().unwrap();
        }
    }
    
    let mut renderer = Renderer::load(display);
    renderer.load_sprite("mychar".to_string(), "src/test/mychar.png", (16.0, 24.0));
    
    let mut closed = false;
    let mut t = 0.0;
    while !closed
    {
        t += 1.0;
        
        renderer.draw_angle("mychar", 32.0, 32.0, 1.0, (deg2rad(t*0.01*2.0)).cos() as f32, (t*0.01) as f32);
        
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
}