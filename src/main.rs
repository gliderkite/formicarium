use anyhow::Result;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::*;
use std::env;

mod entity;
mod game;

/// The path of the file with the default game configuration.
const GAME_CONFIG_PATH: &str = "conf.json";

fn main() -> Result<()> {
    env_logger::init();

    let conf_path = env::args()
        .nth(1)
        .unwrap_or_else(|| GAME_CONFIG_PATH.to_string());
    let conf = game::Conf::parse(conf_path)
        .map_err(|e| log::warn!("Using default configuration: {}", e))
        .unwrap_or_default();

    log::info!("Building game context");
    let (width, height) = conf.size().into();
    let (ctx, events_loop) = &mut ContextBuilder::new("ants", "Marco Conte")
        .window_setup(WindowSetup::default().title("Formicarium!"))
        .window_mode(WindowMode::default().dimensions(width, height))
        .build()?;

    let context = game::Context::with_context(conf, ctx)?;
    let state = &mut game::State::new(&context)?;
    log::info!("Running game loop..");
    event::run(ctx, events_loop, state)?;

    Ok(())
}
