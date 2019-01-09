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
    
    let image = image::load(BufReader::new(File::open("src/test/mychar.png").unwrap()), image::ImageFormat::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let texture = &SrgbTexture2d::new(&display, image).unwrap();
    
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
        #version 330
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 matrix_view;
        uniform mat4 matrix_command;
        uniform mat4 matrix_sprite;
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix_view * matrix_command * vec4(position, 0.0, 1.0);
        }
    "#;
    
    let fragment_shader_src = r#"
        #version 330
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            color = texture(tex, v_tex_coords);
            if(color.a <= 0.5)
                discard;
            
        }
    "#;
    
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    
    struct DrawEvent<'a> {
        matrix : [[f32; 4]; 4],
        texture: &'a SrgbTexture2d
    }
    
    impl<'a> DrawEvent<'a> {
        fn draw_centered(x : f32, y : f32, texture : &'a SrgbTexture2d) -> DrawEvent<'a>
        {
            let dims = texture.dimensions();
            let x_dim = dims.0 as f32;
            let y_dim = dims.1 as f32;
            DrawEvent::draw(x_dim/2.0, y_dim/2.0, x, y, texture)
        }
        fn draw(xorigin : f32, yorigin : f32, x : f32, y : f32, texture : &'a SrgbTexture2d) -> DrawEvent<'a>
        {
            DrawEvent::draw_scaled(xorigin, yorigin, x, y, 1.0, 1.0, texture)
        }
        fn draw_scaled(xorigin : f32, yorigin : f32, x : f32, y : f32, xscale : f32, yscale : f32, texture : &'a SrgbTexture2d) -> DrawEvent<'a>
        {
            DrawEvent::draw_angle(xorigin, yorigin, x, y, xscale, yscale, 0.0, texture)
        }
        fn draw_angle(xorigin : f32, yorigin : f32, x : f32, y : f32, xscale : f32, yscale : f32, angle : f32, texture : &'a SrgbTexture2d) -> DrawEvent<'a>
        {
            let angle_radians = angle as f64 * std::f64::consts::PI / 360.0;
            let angle_cos = angle_radians.cos() as f32;
            let angle_sin = angle_radians.sin() as f32;
            let dims = texture.dimensions();
            let x_dim = dims.0 as f32;
            let y_dim = dims.1 as f32;
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
            //let matrix = matrix_pos;
            DrawEvent{matrix, texture}
        }
    }
    
    let mut closed = false;
    let mut t = 0.0;
    while !closed
    {
        t += 1.0;
        let draw_events = vec!(DrawEvent::draw_angle(16.0, 24.0, 32.0, 32.0, 1.0, (t*0.1/360.0f32).cos(), t*0.01, &texture));
        
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
            let uniforms = uniform! {
                matrix_view: matrix_view,
                matrix_command: event.matrix,
                tex: Sampler::new(event.texture).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
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