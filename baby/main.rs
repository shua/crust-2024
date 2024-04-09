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
        .insert_state(AppState::Intro)
        .add_systems(
            OnEnter(AppState::Intro),
            (intro::setup, intro::setup_anim).chain(),
        )
        .add_systems(
            Update,
            (
                intro::sequence_cues.run_if(in_state(AppState::Intro)),
                intro::sequence_camera.run_if(in_state(AppState::Intro)),
                intro::animate_texture.run_if(in_state(AppState::Intro)),
            ),
        )
        .add_systems(
            PostUpdate,
            (intro::draw_debug.run_if(in_state(AppState::Intro))),
        )
        .add_systems(OnExit(AppState::Intro), intro::cleanup)
        .run();
}
