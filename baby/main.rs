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
                intro::check_kbd.run_if(in_state(AppState::Intro)),
                intro::sequence_cues.run_if(in_state(AppState::Intro)),
                intro::sequence_camera.run_if(in_state(AppState::Intro)),
                intro::animate_texture.run_if(in_state(AppState::Intro)),
            ),
        )
        .add_systems(
            PostUpdate,
            intro::draw_debug.run_if(in_state(AppState::Intro)),
        )
        .add_systems(OnExit(AppState::Intro), intro::cleanup)
        // Game
        .add_systems(
            OnEnter(AppState::Game),
            (level::setup, level::setup_graphics).chain(),
        )
        // I don't know how to conditionally add this
        .add_event::<level::Quit>()
        .add_systems(
            Update,
            (
                level::check_kbd,
                level::check_collide,
                level::update_movement,
                level::update_camera,
            )
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(
            Update,
            (level::check_mouse, level::on_quit, level::animate_texture)
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(
            PostUpdate,
            level::draw_debug.run_if(in_state(AppState::Game)),
        )
        .run();
}
