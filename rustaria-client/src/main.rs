use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use eyre::Result;
use glfw::{Action, Context, Key, Modifiers, SwapInterval, WindowEvent};
use mlua::Lua;
use structopt::StructOpt;
use tracing::{debug, info};

use opengl_render::OpenGlBackend;
use rustaria::api::Rustaria;
use rustaria::network::{PacketDescriptor, PacketOrder, PacketPriority};
use rustaria::network::packet::{ClientPacket, ServerPacket};
use rustaria::types::ChunkPos;

use crate::network::{Client, RemoteServerCom, ServerCom};
use crate::render::RustariaRenderer;

mod network;
mod render;

const DEBUG_MOD: Modifiers = Modifiers::from_bits_truncate(glfw::ffi::MOD_ALT | glfw::ffi::MOD_SHIFT);


#[derive(Debug, StructOpt)]
#[structopt(name = "rustaria-client", about = "The interactive client of Rustaria")]
struct Opt {
    #[structopt(flatten)]
    inner: rustaria::opt::Opt,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    debug!(?opt, "Got command-line args");
    rustaria::init(opt.inner.verbosity)?;


    let title = &*format!("Rustaria Client v{}", env!("CARGO_PKG_VERSION"));
    info!(title);

    info!(target: "render", "Launching GLFW");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    info!(target: "render", "Creating Window");
    let (mut window, events) = glfw.create_window(900, 600, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    info!(target: "render", "Loading OpenGL backend");
    window.set_key_polling(true);
    window.set_size_polling(true);
    window.set_scroll_polling(true);
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(1));

    let mut renderer = RustariaRenderer::new(&glfw, &window);


    let mut zoom = 0.0;
    let mut w = false;
    let mut a = false;
    let mut s = false;
    let mut d = false;

    let mut x = 0.0;
    let mut y = 0.0;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Size(width, height) => {
                    renderer.resize(width as u32, height as u32);
                }
                WindowEvent::Scroll(x, y) => {
                    zoom -= y * 0.01;
                    info!("{zoom}");
                    renderer.world_renderer.qi_u_zoom.set_value(zoom as f32);
                }
                WindowEvent::Key(Key::Q, _, Action::Press, DEBUG_MOD) => window.set_should_close(true),
                WindowEvent::Key(Key::W, _, Action::Press, DEBUG_MOD) => renderer.wireframe = !renderer.wireframe,
                WindowEvent::Key(Key::W, _, action, _) => w = action != Action::Release,
                WindowEvent::Key(Key::A, _, action, _) => a = action != Action::Release,
                WindowEvent::Key(Key::S, _, action, _) => s = action != Action::Release,
                WindowEvent::Key(Key::D, _, action, _) => d = action != Action::Release,
                _ => {}
            }
        }

        x += (d as i8 - a as i8) as f32 * 0.008;
        y += (w as i8 - s as i8) as f32 * 0.008;
        // render stuff
        renderer.draw(x, y)?;
        window.swap_buffers();
    }


    Ok(())
}


pub fn create_window() {}
