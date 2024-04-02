use bevy::{
    math::bounding::{Aabb2d, AabbCast2d},
    prelude::*,
};

#[derive(Component)]
struct Control;
#[derive(Component)]
struct Collide;
#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_systems(Startup, setup_graphics)
        .add_systems(
            Update,
            (
                check_keys,    //
                check_collide, //
                update_movement,
            )
                .chain(),
        )
        .run();
}

const MAP: [u8; 8 * 8] = [
    1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 0, 0, 0, 0, 0, 0, 1, // 2
    1, 0, 0, 0, 0, 0, 0, 1, // 3
    1, 0, 0, 0, 0, 0, 0, 1, // 4
    1, 0, 0, 0, 0, 0, 0, 1, // 5
    1, 0, 0, 0, 0, 0, 0, 1, // 6
    1, 0, 0, 0, 0, 0, 0, 1, // 7
    1, 1, 1, 1, 1, 1, 1, 1, // 8
];

fn setup_graphics(mut command: Commands) {
    command.spawn(Camera2dBundle::default());
    command.spawn((
        Control,
        Collide,
        Velocity::default(),
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

    for y in 0..8 {
        for x in 0..8 {
            if MAP[y * 8 + x] == 1 {
                command.spawn((
                    Collide,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.5, 0.2, 0.7),
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(
                                (x as f32 - 4.) * 50.,
                                (y as f32 - 4.) * 50.,
                                0.0,
                            ),
                            scale: Vec3::new(50., 50., 1.),
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
        }
    }
}

fn check_keys(
    kbd: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut ctl: Query<&mut Velocity, With<Control>>,
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

    let v = Vec2::new(vx, vy) * time.delta_seconds() * 500.;
    for mut c in &mut ctl {
        c.0 = v;
    }
}

// the intent is to cast the ctl's aabb along ctl's velocity and check for any collisions
// if there are any collisions, then reduce velocity until there aren't
//
// this is not working correctly as it sees collisions where it shouldn't
fn check_collide(
    mut ctl: Query<(Entity, &Transform, &mut Velocity, &mut Sprite), With<Control>>,
    col: Query<(Entity, &Transform), With<Collide>>,
) {
    for (e, t, mut v, mut s) in &mut ctl {
        if v.0 == Vec2::ZERO {
            continue;
        }

        let aabb = Aabb2d::new(t.translation.xy(), t.scale.xy() / 2.);
        let mut aabb_cast = AabbCast2d::new(
            aabb,
            t.translation.xy(),
            Direction2d::new(**v).unwrap(),
            v.length(),
        );
        let prev_max = aabb_cast.ray.max;
        for (ec, col) in &col {
            if ec == e {
                continue;
            }
            let col_aabb = Aabb2d::new(col.translation.xy(), col.scale.xy() / 2.);
            if let Some(dist) = aabb_cast.aabb_collision_at(col_aabb) {
                println!(
                    "collision: {:?} + {:?} with {:?}",
                    t.translation, v.0, col.translation
                );
                aabb_cast.ray.max = dist;
            }
        }

        if aabb_cast.ray.max < prev_max {
            // v.0 = v.0 * (aabb_cast.ray.max / v.length());
            s.color = Color::rgb(1., 0., 0.);
        } else {
            s.color = Color::rgb(0., 0., 1.);
        }
    }
}

fn update_movement(mut movers: Query<(&mut Transform, &Velocity)>) {
    for (mut t, v) in &mut movers {
        t.translation += Vec3::new(v.0.x, v.0.y, 0.);
    }
}
