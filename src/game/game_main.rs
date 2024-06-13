
use winit::keyboard::KeyCode;

use crate::{
    framework::{get_debug_mode, set_debug_mode, DebugMode},
    managers::{
        input::{self, InputEventType}, scripting::lua::LuaSystem, systems::add_system
    },
    systems::player_manager::PlayerManager
};

pub fn start() {
    input::new_bind(
        "debug_toggle",
        vec![InputEventType::Key(KeyCode::Backquote)],
    );

    //add_system(Box::new(TestSystem::new()));
    add_system(Box::new(PlayerManager::new()));
    //add_system(Box::new(WorldGenerator::new()));
    add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()));
    add_system(Box::new(LuaSystem::new("world_generator", "scripts/lua/world_generation.lua").unwrap()));
    add_system(Box::new(LuaSystem::new("tile1", "scripts/lua/tile1.lua").unwrap()));
}

pub fn update() {
    if input::is_bind_pressed("debug_toggle") {
        match get_debug_mode() {
            DebugMode::None => set_debug_mode(DebugMode::ShowFps),
            _ => {
                set_debug_mode(DebugMode::None);
            }
        }
    }
}

//pub fn render() {}
