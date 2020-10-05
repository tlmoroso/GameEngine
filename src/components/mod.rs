use crate::components::animation::Animation;
use crate::components::mesh_graphic::MeshGraphic;
use crate::components::player_control::PlayerControl;
use crate::components::position::Position;
use crate::components::text_display::TextDisplay;

pub mod player_control;
pub mod position;
pub mod text_display;
pub mod mesh_graphic;
pub mod animation;

pub const COMPONENTS_LOAD_PATH: &str = "components";

#[derive(Debug)]
pub enum ComponentType {
    Animation(Animation),
    MeshGraphic(MeshGraphic),
    PlayerControl(PlayerControl),
    Position(Position),
    TextDisplay(TextDisplay),
}