use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;
// surely this should be wide enough
const PILLARBOX_WIDTH: f32 = 2000.;

mod intro;
mod level;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Intro,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Baby".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        // Intro
        .insert_state(AppState::Intro)
        .add_systems(
            OnEnter(AppState::Intro),
            (intro::setup, intro::setup_anim).chain(),
        )
        .add_systems(
            Update,
            (
                intro::sequence_cues,
                intro::sequence_camera,
                intro::animate_texture,
            )
                .run_if(in_state(AppState::Intro)),
        )
        .add_systems(
            PostUpdate,
            intro::draw_debug.run_if(in_state(AppState::Intro)),
        )
        .add_systems(OnExit(AppState::Intro), intro::cleanup)
        // Game
        .add_plugins(level::DebugGamePlugin)
        .insert_resource(level::PhysicsTick(0.))
        .insert_resource(level::TileTypes(vec![default()]))
        .add_systems(OnEnter(AppState::Game), level::setup)
        .add_systems(
            Update,
            (
                level::check_kbd,
                level::check_collide,
                level::update_movement,
                level::pan_camera,
            )
                .run_if(in_state(AppState::Game))
                .chain(),
        )
        .add_systems(
            Update,
            level::animate_texture.run_if(in_state(AppState::Game)),
        )
        .run();
}
