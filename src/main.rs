#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use gfx::Device;
use gfx::traits::FactoryExt;
use gfx_window_glutin as gfx_glutin;
use glutin::VirtualKeyCode;
use glutin::WindowEvent::*;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        image: gfx::TextureSampler<[f32; 4]> = "t_Image",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub fn main() {
    // Start a new glutin event loop
    let events_loop = glutin::EventsLoop::new();

    // Define a window builder
    let builder = glutin::WindowBuilder::new()
        .with_title("gfx-rs-image-test".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    // Initialize glutin
    let (
        window,
        mut device,
        mut factory,
        main_color,
        mut main_depth
    ) = gfx_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);

    // Create the command encoder
    let mut encoder: gfx::Encoder<_, _> = factory
        .create_command_buffer()
        .into();

    // Create a shader pipeline
    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/screen.glslv")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/screen.glslf")),
        pipe::new(),
    ).unwrap();

    // Create the vertices and indices, define the vertex buffer
    let (vertices, indices) = create_vertices_indices();
    let (vertex_buffer, mut slice) = factory
        .create_vertex_buffer_with_slice(&vertices, &*indices);

    // Load the screen texture
    let texture = load_texture(&mut factory, "assets/test.png");

    // Create an image sampler
    let sampler = factory.create_sampler_linear();

    // Build pipe data
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        image: (texture, sampler),
        out: main_color,
    };

    // Rendering flags
    let mut running = true;
    let mut update = false;
    let mut dimentions = (800.0, 600.0);

    // Keep rendering until we're done
    while running {
        // Update graphics when required
        if update {
            let (vertices, indices) = create_vertices_indices();
            let (vertex_buff, sl) = factory
                .create_vertex_buffer_with_slice(&vertices, &*indices);

            // Redefine the vertex buffer and slice
            data.vbuf = vertex_buff;
            slice = sl;

            // We've successfully updated
            update = false
        }

        // Poll vor events
        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            match event {
                // Stop running when escape is pressed
                KeyboardInput(_, _, Some(VirtualKeyCode::Escape), _)
                | Closed => running = false,

                // Update the view when the window is resized
                Resized(w, h) => {
                    gfx_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    dimentions = (w as f32, h as f32);
                    update = true
                },

                _ => (),
            }
        });

        // Clear the buffer
        encoder.clear(&data.out, BLACK);

        // Draw through the pipeline
        encoder.draw(&slice, &pso, &data);

        encoder.flush(&mut device);

        // Swap the frame buffers
        window.swap_buffers().unwrap();

        device.cleanup();
    }
}

/// Load a texture from the given `path`.
fn load_texture<F, R>(factory: &mut F, path: &str)
    -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
{
    // Ope the image from the given path
    let img = image::open(path).unwrap().to_rgba();
    let (width, height) = img.dimensions();

    // Define the texture kind
    let kind = gfx::texture::Kind::D2(
        width as u16,
        height as u16,
        gfx::texture::AaMode::Single
    );

    // Create a GPU texture
    let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(kind, &[&img]).unwrap();

    view
}

/// Create the vertices and indices for a full screen quad.
pub fn create_vertices_indices() -> (Vec<Vertex>, Vec<u16>) {
    // Build the vertices
    let vertices = vec![
        Vertex { pos: [ 1f32, -1f32], uv: [1.0, 1.0] },
        Vertex { pos: [-1f32, -1f32], uv: [0.0, 1.0] },
        Vertex { pos: [-1f32,  1f32], uv: [0.0, 0.0] },
        Vertex { pos: [ 1f32,  1f32], uv: [1.0, 0.0] },
    ];

    // Define the vertex indices
    let indices = vec![
        0, 1, 2,
        2, 3, 0,
    ];

    (vertices, indices)
}