extern crate ggez;
mod game;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder, GameResult,
};

use std::path;

fn main() -> GameResult {
    let win_mode = WindowMode::default().dimensions(1000., 800.);

    let win_setup = WindowSetup::default().title("King of the hill & three check mod");

    let mut asset_path = path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    asset_path.push("src");
    asset_path.push("assets");

    let (mut ctx, event_loop) = ContextBuilder::new("King of the hill & three check mod", "rejnhed")
        .window_setup(win_setup)
        .window_mode(win_mode)
        .add_resource_path(asset_path)
        .build()
        .unwrap();
        
    let game = game::RChess::new(&mut ctx)?;

    event::run(ctx, event_loop, game)
}
