mod entities;
mod components;
mod systems;
mod scenes;
mod game;
mod input;
mod load;
mod globals;


use coffee::graphics::WindowSettings;
use coffee::{Game, Result};
use game::*;

fn main() -> Result<()> {
    MyGame::run(WindowSettings {
        title: String::from("A caffeinated game"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
