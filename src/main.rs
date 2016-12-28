#[macro_use]
extern crate glium;

extern crate rustc_serialize;
extern crate docopt;

use glium::backend::Facade;

const NAME: &'static str = "minrs";

const USAGE: &'static str = r#"
I kept dreaming of a world I thought I'd never see

Usage:
  minrs <file>
  minrs (-h | --help)
  minrs --version

Options:
  -v, --verbose  Show debug info on stdout.
  -h, --help     Show this screen.
  --version      Show version.
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
  arg_file: String,
  flag_verbose: bool,
}

fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new()
        .with_title(NAME)
        .with_vsync()
        .build_glium()
        .unwrap();
    let version = display.get_opengl_version();
    println!("OpenGL version {:?}", version);
    let (width, height) = display.get_context().get_framebuffer_dimensions();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let mut shape = vec![];
    let half_width = width as f32 / 2f32;
    let half_height = height as f32 / 2f32;
    for y in 0..height {
        for x in 0..width {
            let xx = (x as f32 - half_width) / half_width;
            let yy = (y as f32 - half_height) / half_height;
            shape.push(Vertex{position: [xx, yy]});
        }
    }
    println!("shape size: {:?}", shape.len());

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let texture = load_file_1d(&display, width, height, args.arg_file.as_str()).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
    let uniforms = uniform! {
        tex: &texture
    };

    let program = program!(&display,
                           140 => {
                               point_size: true,
                               vertex: "
        #version 140

        in vec2 position;
        out vec2 pos;

        void main() {
            gl_PointSize = 1;
            gl_Position = vec4(position, 0.0, 1.0);
            pos = position;
        }
        ",
                               fragment: "
        #version 140

        uniform sampler1D tex;

        in vec2 pos;
        out vec4 color;

        void main() {
            vec4 c = texture(tex, pos.x);
            color = vec4(pos, c.r, 1.0);
        }
        ",
                           }).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}

#[derive(Debug)]
enum Load1DError {
    Io(std::io::Error),
    Gl(glium::texture::TextureCreationError),
}

fn load_file_1d<F: ?Sized>(display: &F, width: u32, height: u32, path: &str)
    -> Result<glium::texture::Texture1d, Load1DError> where F: Facade + std::marker::Sized {
    let read_bytes = std::cmp::min(1024, width * height);
    println!("trying to read {:?} of {:?}", read_bytes, path);

    let mut buffer = Vec::new();
    let f = try!(std::fs::File::open(path).map_err(Load1DError::Io));
    let mut chunk = f.take(read_bytes as u64);
    use std::io::Read;
    let bytes_read = try!(chunk.read_to_end(&mut buffer).map_err(Load1DError::Io)) as u32;
    println!("read {:?}", bytes_read);

    use glium::texture::pixel_buffer::PixelBuffer;
    let pixelbuffer = PixelBuffer::new_empty(display, bytes_read as usize);
    pixelbuffer.write(buffer.as_slice());
    println!("pixelbuffer size: {:?}", pixelbuffer.get_size());

    use glium::texture::Texture1d;
    let texture = try!(Texture1d::empty_with_format(display,
                                                    glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                                    glium::texture::MipmapsOption::NoMipmap,
                                                    bytes_read)
                       .map_err(Load1DError::Gl));

    texture.main_level().raw_upload_from_pixel_buffer(pixelbuffer.as_slice(), 0..bytes_read, 0..1, 0..1);
    println!("texture info: {:?} {:?} {:?} {:?} {:?} {:?}"
             ,texture.get_width()
             ,texture.get_height()
             ,texture.get_depth()
             ,texture.kind()
             ,texture.get_texture_type()
             ,texture.get_mipmap_levels()
            );
    Ok(texture)
}
