use crate::scenes::{SceneStack, SCENE_STACK_FILE_ID};
use crate::input::CustomInput;
use crate::components::{player_control::PlayerControl, position::Position, text_display::TextDisplay, mesh_graphic::MeshGraphic, animation::Animation};

use coffee::graphics::{Color, Frame, Window};
use coffee::load::Task;
use coffee::{Game, Timer};

use specs::World;
use specs::WorldExt;
use crate::load::{JSON_FILE, LOAD_PATH, load_json};
use std::sync::{Arc, RwLock};
use crate::globals::{FontDict, FONT_DICT_FILE_ID};

pub struct MyGame {
    scene_stack: SceneStack,
    ecs: Arc<RwLock<World>>,
}

impl MyGame {
    fn register_components(ecs: &mut World) {
        ecs.register::<Position>();
        ecs.register::<PlayerControl>();
        ecs.register::<TextDisplay>();
        ecs.register::<MeshGraphic>();
        ecs.register::<Animation>();
    }
}

impl MyGame {
    pub fn load_game(&mut self, window: &mut Window) {
        let font_dict_json_value = load_json([LOAD_PATH, FONT_DICT_FILE_ID, JSON_FILE].join(""));
        if font_dict_json_value.loadable_type.eq(FONT_DICT_FILE_ID) {
            let font_dict = FontDict::load(self.ecs.clone(), Arc::new(RwLock::new(window)), font_dict_json_value.other_value)
                .run(window.gpu()).unwrap();
            self.ecs
                .write()
                .expect("ERROR: RwLock poisoned for ecs in Game::interact")
                .insert::<FontDict>(font_dict);
        } else { panic!(format!("ERROR: font_dict_json_value == {} instead of {}", font_dict_json_value.loadable_type, FONT_DICT_FILE_ID)); }

        let scene_stack_json_value = load_json([LOAD_PATH, SCENE_STACK_FILE_ID, JSON_FILE].join(""));
        if scene_stack_json_value.loadable_type.eq(SCENE_STACK_FILE_ID) {
            self.scene_stack = SceneStack::load(self.ecs.clone(), Arc::new(RwLock::new(window)), scene_stack_json_value.other_value)
                .run(window.gpu()).unwrap()
        } else { panic!(format!("ERROR: scene_stack_json_value == {} instead of {}", scene_stack_json_value.loadable_type, SCENE_STACK_FILE_ID)); }

    }
}

impl Game for MyGame {
    type Input = CustomInput;
    type LoadingScreen = ();

    fn load(window: &Window) -> Task<MyGame> {
        let mut world = World::new();
        MyGame::register_components(&mut world);

        let scene_stack = SceneStack{stack: vec![], loaded: false };

        Task::succeed(|| MyGame {
            scene_stack,
            ecs: Arc::new(RwLock::new(world)),
        })
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        frame.clear(Color::BLACK);
        self.scene_stack.draw(self.ecs.clone(), frame, timer);
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        if self.scene_stack.loaded {
            self.scene_stack.interact(self.ecs.clone(), input, Arc::new(RwLock::new(window)));
        } else {
            self.load_game(window);
        }
    }

    fn update(&mut self, _window: &Window) {
        self.scene_stack.update(self.ecs.clone());
    }
}
