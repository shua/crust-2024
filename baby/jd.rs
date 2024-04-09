use bevy::prelude::*;

mod level;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Baby".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_event::<level::Quit>()
        .add_systems(Startup, (level::setup, level::setup_graphics).chain())
        .add_systems(
            Update,
            (
                level::check_kbd,
                level::check_collide,
                level::update_movement,
                level::update_camera,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (level::check_mouse, level::on_quit, level::animate_texture),
        )
        .add_systems(PostUpdate, level::draw_debug)
        .run();
}
