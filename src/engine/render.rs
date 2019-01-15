use glium::{Surface, implement_vertex, uniform};
use glium::texture::SrgbTexture2d;
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};

use super::*;

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

fn deg2rad(x : f64) -> f64
{
    x * std::f64::consts::PI / 360.0
}

#[derive(Copy, Clone)]
pub (super) struct Vertex {
    position: [f32; 2]
}

implement_vertex!(Vertex, position);

pub (super) struct SpriteImage {
    origin: (f64, f64),
    topleft: (f64, f64),
    bottomright: (f64, f64),
}

impl SpriteImage {
    pub (super) fn basic(origin: (f64, f64), tex : &SrgbTexture2d) -> SpriteImage
    {
        SpriteImage{origin, topleft: (0.0, 0.0), bottomright: (tex.width() as f64, tex.height() as f64)}
    }
    pub (super) fn extended(origin: (f64, f64), topleft: (f64, f64), bottomright : (f64, f64)) -> SpriteImage
    {
        SpriteImage{origin, topleft, bottomright}
    }
}

pub (super) struct SpriteSheet {
    images: Vec<SpriteImage>,
    texture: SrgbTexture2d,
}

pub (super) struct DrawEvent {
    matrix: [[f32; 4]; 4],
    spritesheet: u64,
    imageindex: u64
}


impl Engine {
    pub (super) fn build_glprogram(display : &glium::Display, program_path : &String) -> glium::Program
    {
        let vertex_shader_src = load_string(program_path, "data/glsl/vertex.glsl").unwrap();
        let fragment_shader_src = load_string(program_path, "data/glsl/fragment.glsl").unwrap();
        let glprogram = glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
        
        glprogram
    }
    pub (super) fn build_vertex_buffer(display : &glium::Display) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices)
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
    
    pub (super) fn load_sprite(&mut self, fname : &str, origin : (f64, f64)) -> u64
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
    
    pub (super) fn load_sprite_with_subimages(&mut self, fname : &str, images : Vec<SpriteImage>) -> u64
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
    
    pub (super) fn draw_sprite(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32)
    {
        self.draw_sprite_scaled(spriteindex, imageindex, x, y, 1.0, 1.0)
    }
    pub (super) fn draw_sprite_scaled(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32, xscale : f32, yscale : f32)
    {
        self.draw_sprite_angled(spriteindex, imageindex, x, y, xscale, yscale, 0.0)
    }
    pub (super) fn draw_sprite_angled(&mut self, spriteindex : u64, imageindex : u64, x : f32, y : f32, xscale : f32, yscale : f32, angle : f32)
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
    pub (super) fn draw_sprite_transformed(&mut self, spriteindex : u64, imageindex : u64, matrix : [[f32; 4]; 4])
    {
        self.draw_events.push(DrawEvent{matrix, spritesheet : spriteindex, imageindex : imageindex})
    }
    
    pub (crate) fn render(&mut self)
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
