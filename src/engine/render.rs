use glium::{implement_vertex, uniform};
use glium::texture::{SrgbTexture2d, DepthTexture2d};
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
use std::rc::Rc;

use glium::Surface as _;

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

#[derive(Debug, Copy, Clone)]
pub (super) struct Vertex {
    position: [f32; 2]
}

implement_vertex!(Vertex, position);

#[derive(Debug)]
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

#[derive(Debug)]
pub (super) struct SpriteSheet {
    images: Vec<SpriteImage>,
    texture: SrgbTexture2d,
}

pub (super) struct Surface {
    dims : (u32, u32),
    rgba : SrgbTexture2d,
    depth : DepthTexture2d,
}

impl Surface {
    pub (crate) fn new(display : &glium::Display, (w, h) : (u32, u32)) -> Surface
    {
        let rgba = SrgbTexture2d::empty(display, w, h).unwrap();
        let depth = DepthTexture2d::empty(display, w, h).unwrap();
        let mut target = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(display, &rgba, &depth).unwrap();
        target.clear_color_srgb_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        Surface{dims : (w, h), rgba, depth}
    }
    pub (crate) fn as_framebuffer<'a>(&'a self, display : &glium::Display) -> glium::framebuffer::SimpleFrameBuffer<'a>
    {
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(display, &self.rgba, &self.depth).unwrap()
    }
    fn clear_color_infinite_depth(&self, display : &glium::Display, color : (f32, f32, f32, f32))
    {
        self.as_framebuffer(display).clear_color_srgb_and_depth(color, 1.0);
    }
}

#[derive(Debug, Clone)]
struct TextDrawData {
    tex_coords: glyph_brush::rusttype::Rect<f32>,
    draw_coords: glyph_brush::rusttype::Rect<i32>,
    color : [f32; 4],
}

impl TextDrawData
{
    pub (crate) fn new(quad_data : glyph_brush::GlyphVertex) -> TextDrawData
    {
        TextDrawData{tex_coords : quad_data.tex_coords, draw_coords : quad_data.pixel_coords, color : quad_data.color}
    }
}

pub (super) struct TextSystem {
    glyph_brush : glyph_brush::GlyphBrush<'static, TextDrawData>,
    texture : glium::texture::Texture2d,
    texture_dimensions : (u32, u32),
    cached_draw : Vec<TextDrawData>,
    display : glium::Display,
}

impl TextSystem
{
    pub (crate) fn new(display : &glium::Display) -> TextSystem
    {
        use glyph_brush::GlyphBrushBuilder;
        use glium::texture::Texture2d;
        use glium::texture::UncompressedFloatFormat::U8U8U8U8;
        use glium::texture::MipmapsOption;
        
        let font: &[u8] = include_bytes!("../../data/font/Chivo-Regular.ttf");
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font).build();
        
        let texture_dimensions = glyph_brush.texture_dimensions();
        let texture = Texture2d::empty_with_format(display, U8U8U8U8, MipmapsOption::NoMipmap, texture_dimensions.0, texture_dimensions.1).unwrap();
        
        TextSystem{glyph_brush, texture, texture_dimensions, cached_draw : Vec::new(), display: display.clone()}
    }
    fn update_texture(texture : &mut glium::texture::Texture2d, rect : glyph_brush::rusttype::Rect<u32>, tex_data : &[u8])
    {
        let dims = rect.max - rect.min;
        let mut buffer = vec!(vec!((0u8, 0u8, 0u8, 0u8); dims.x as usize); dims.y as usize);
        let mut i = 0;
        for y in 0..dims.y
        {
            for x in 0..dims.x
            {
                buffer[y as usize][x as usize] = (255, 255, 255, tex_data[i]);
                i += 1;
            }
        }
        texture.write
        (
            glium::Rect
            {
                left : rect.min.x,
                bottom : rect.min.y,
                width : dims.x,
                height : dims.y
            },
            buffer
        );
    }
    fn resize_texture(&mut self, new_size : (u32, u32))
    {
        use glium::texture::Texture2d;
        use glium::texture::UncompressedFloatFormat::U8U8U8U8;
        use glium::texture::MipmapsOption;
        self.texture_dimensions = new_size;
        self.glyph_brush.resize_texture(self.texture_dimensions.0, self.texture_dimensions.1);
        self.texture = Texture2d::empty_with_format(&self.display, U8U8U8U8, MipmapsOption::NoMipmap, self.texture_dimensions.0, self.texture_dimensions.1).unwrap();
    }
    pub (crate) fn draw_text(&mut self, parent : &Engine, text : &String, x : f32, y : f32, w : f32, h : f32, size : f32, color : [f32; 4])
    {
        self.glyph_brush.queue(glyph_brush::Section {
            text,
            screen_position : (x, y),
            bounds : (w, h),
            scale : glyph_brush::rusttype::Scale::uniform(size),
            color,
            ..glyph_brush::Section::default()
        });
        
        let mut succeeded = false;
        while !succeeded
        {
            succeeded = true;
            match self.process_queue()
            {
                Ok(glyph_brush::BrushAction::Draw(quads)) =>
                {
                    self.draw_quads(&quads, parent);
                    self.cached_draw = quads;
                }
                Ok(glyph_brush::BrushAction::ReDraw) =>
                {
                    self.draw_quads(&self.cached_draw, parent);
                }
                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) =>
                {
                    succeeded = false;
                    self.resize_texture(suggested);
                }
            }
        }
    }
    fn process_queue(&mut self) -> Result<glyph_brush::BrushAction<TextDrawData>, glyph_brush::BrushError> 
    {
        let texture = &mut self.texture;
        self.glyph_brush.process_queued(
            |rect, tex_data| TextSystem::update_texture(texture, rect, tex_data),
            TextDrawData::new
        ) 
    }
    fn draw_quads(&self, quads : &Vec<TextDrawData>, parent : &Engine)
    {
        let mut target = parent.get_real_draw_target();
        for quad in quads
        {
            let tex_rect = quad.tex_coords;
            
            let draw_rect = quad.draw_coords;
            let draw_size = draw_rect.max - draw_rect.min;
            
            let matrix_origin = [
                [draw_size.x as f32, 0.0, 0.0, 0.0],
                [0.0, draw_size.y as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            let event_matrix = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [draw_rect.min.x as f32, draw_rect.min.y as f32, 0.0, 1.0],
            ];
            let matrix_command = m4mult(&event_matrix, &matrix_origin);
            
            let uniforms = uniform! {
                matrix_view : parent.matrix_view.clone(),
                matrix_command : matrix_command,
                tex_topleft : [tex_rect.min.x, tex_rect.min.y],//quad.0.min.x, quad.0.min.y],
                tex_bottomright : [tex_rect.max.x, tex_rect.max.y],//quad.0.max.x, quad.0.max.y],
                color_multiply : quad.color,
                tex : Sampler::new(&self.texture),
            };
            target.draw(&parent.vertex_buffer, &parent.indices, &parent.current_program, &uniforms, &glium::DrawParameters
            {
                blend : glium::Blend::alpha_blending(),
                ..Default::default()
            }).unwrap();
        }
    }
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
    
    pub (super) fn load_program(&mut self, filename_vertex : &str, filename_fragment : &str) -> Result<u64, String>
    {
        let vertex_shader_src = load_string(&self.program_path, filename_vertex)?;
        let fragment_shader_src = load_string(&self.program_path, filename_fragment)?;
        let glprogram = glium::Program::from_source(&self.display, &vertex_shader_src, &fragment_shader_src, None).or_else(|x| Err(format!("failed to compile program: {}", x)))?;
        
        let index = self.program_index_counter;
        self.programs.insert(index, Rc::new(glprogram));
        self.program_index_counter += 1;
        Ok(index)
    }
    
    pub (super) fn set_program(&mut self, program_id : u64) -> Result<(), String>
    {
        self.current_program = Rc::clone(self.programs.get(&program_id).ok_or_else(|| "no such gl program".to_string())?);
        Ok(())
    }
    
    pub (super) fn reset_program(&mut self)
    {
        self.current_program = Rc::clone(&self.default_program);
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
    
    pub (super) fn draw_text(&mut self, text : &String, x :f32, y : f32, w : f32, h : f32, size : f32, color : [f32; 4])
    {
        self.text_system.borrow_mut().draw_text(&self, &text, x, y, w, h, size, color);
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
        let spritesheet = self.sprites.get(&spriteindex).unwrap();
        let texture = &spritesheet.texture;
        
        let tex_w = texture.width() as f32;
        let tex_h = texture.height() as f32;
        
        let image = spritesheet.images.get(imageindex as usize % spritesheet.images.len()).unwrap();
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
        let matrix_command = m4mult(&matrix, &matrix_origin);
        
        let uniforms = uniform! {
            matrix_view : self.matrix_view,
            matrix_command : matrix_command,
            tex_topleft : [image.topleft.0 as f32 / tex_w, image.topleft.1 as f32 / tex_h],
            tex_bottomright : [image.bottomright.0 as f32 / tex_w, image.bottomright.1 as f32 / tex_h],
            color_multiply : [1.0, 1.0, 1.0, 1.0f32],
            tex : Sampler::new(texture).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
        };
        self.get_real_draw_target().draw(&self.vertex_buffer, &self.indices, &self.current_program, &uniforms, &glium::DrawParameters
        {
            blend : glium::Blend::alpha_blending(),
            ..Default::default()
        }).unwrap();
    }
    
    pub (crate) fn get_real_draw_target<'a>(&'a self) -> glium::framebuffer::SimpleFrameBuffer<'a>
    {
        let target = match self.surface_target.last()
        {
            Some(index) => self.surfaces.get(&index).unwrap(),
            None => self.default_surface.as_ref().unwrap()
        };
        target.as_framebuffer(&self.display)
    }
    
    pub (crate) fn render_begin(&mut self)
    {
        let target = self.display.draw();
        let dims = target.get_dimensions();
        
        if let Some(default_surface) = &self.default_surface
        {
            if default_surface.dims == dims
            {
                default_surface.clear_color_infinite_depth(&self.display, (0.5, 0.5, 0.5, 1.0));
                self.draw_target = Some(target);
                self.draw_w = dims.0;
                self.draw_h = dims.1;
                self.matrix_view = [
                    [2.0/dims.0 as f32, 0.0, 0.0, 0.0],
                    [0.0, -2.0/dims.1 as f32, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0f32],
                ];
                return;
            }
        }
        self.default_surface =  Some(Surface::new(&self.display, target.get_dimensions()));
        self.default_surface.as_ref().unwrap().clear_color_infinite_depth(&self.display, (0.5, 0.5, 0.5, 1.0));
        self.draw_target = Some(target);
    }
    
    pub (crate) fn render_finish(&mut self)
    {
        let target = self.draw_target.as_mut().unwrap();
        
        let matrix_view = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];
        let matrix_command = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0f32],
        ];
        
        let uniforms = uniform! {
            matrix_view : matrix_view,
            matrix_command : matrix_command,
            tex_topleft : [0.0, 0.0f32],
            tex_bottomright : [1.0, 1.0f32],
            color_multiply : [1.0, 1.0, 1.0, 1.0f32],
            tex : Sampler::new(&self.default_surface.as_ref().unwrap().rgba).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
        };
        target.draw(&self.vertex_buffer, &self.indices, &self.current_program, &uniforms, &glium::DrawParameters
        {
            ..Default::default()
        }).unwrap();
        
        target.set_finish().unwrap();
        self.draw_target = None;
    }
}
