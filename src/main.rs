use bevy::prelude::*;

#[derive(Component)]
struct Control;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_systems(Startup, setup_graphics)
        .add_systems(Update, update)
        .run();
}

fn setup_graphics(mut command: Commands) {
    command.spawn(Camera2dBundle::default());
    command.spawn((
        Control,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1., 0., 0.),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                scale: Vec3::new(50., 50., 1.),
                ..default()
            },
            ..default()
        },
    ));
}

fn update(
    kbd: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut ctl: Query<&mut Transform, With<Control>>,
) {
    if kbd.pressed(KeyCode::Escape) {
        exit.send(bevy::app::AppExit);
    }

    let mut vx = 0.;
    let mut vy = 0.;
    if kbd.pressed(KeyCode::ArrowLeft) {
        vx -= 1.;
    }
    if kbd.pressed(KeyCode::ArrowRight) {
        vx += 1.;
    }
    if kbd.pressed(KeyCode::ArrowUp) {
        vy += 1.;
    }
    if kbd.pressed(KeyCode::ArrowDown) {
        vy -= 1.;
    }

    let v = Vec3::new(vx, vy, 0.) * time.delta_seconds() * 500.;
    for mut c in &mut ctl {
        c.translation += v;
    }
}
