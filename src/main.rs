use glium::{implement_vertex, uniform};
use std::io::BufReader;
use std::fs::File;

fn main()
{
    use glium::{glutin, Surface};
    use glium::texture::SrgbTexture2d;
    
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    
    let image = image::load(BufReader::new(File::open("src/test/mychar.png").unwrap()), image::ImageFormat::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let texture = SrgbTexture2d::new(&display, image).unwrap();
    
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }
    
    implement_vertex!(Vertex, position, tex_coords);
    
    let vertex1 = Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [0.0, 1.0], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [1.0, 0.0], tex_coords: [1.0, 0.0] };
    let vertex4 = Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];
    
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
    
    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 matrix_view;
        uniform mat4 matrix_command;
        uniform mat4 matrix_sprite;
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix_view * matrix_command * matrix_sprite * vec4(position, 0.0, 1.0);
        }
    "#;
    
    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;
    
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    
    struct DrawEvent<'a> {
        matrix : [[f32; 4]; 4],
        texture: &'a SrgbTexture2d
    }
    
    impl<'a> DrawEvent<'a> {
        fn new(x : f32, y : f32, xscale : f32, yscale : f32, texture : &'a SrgbTexture2d) -> DrawEvent<'a>
        {
            let matrix = [
                [xscale, 0.0, 0.0, 0.0],
                [0.0, yscale, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, 0.0, 1.0],
            ];
            DrawEvent{matrix, texture}
        }
    }
    
    let draw_events = vec!(DrawEvent::new(10.0, 10.0, 1.0, 1.0, &texture));
    let mut closed = false;
    while !closed
    {
        let mut target = display.draw();
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
        
        for event in &draw_events
        {
            let dims = event.texture.dimensions();
            let x_dim = dims.0 as f32;
            let y_dim = dims.1 as f32;
            let matrix_sprite = [
                [x_dim, 0.0, 0.0, 0.0],
                [0.0, y_dim, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            let uniforms = uniform! {
                matrix_view: matrix_view,
                matrix_command: event.matrix,
                matrix_sprite: matrix_sprite,
                tex: event.texture,
            };
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        }
        target.finish().unwrap();
        
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