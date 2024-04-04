use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};

#[derive(Component)]
struct Control;
#[derive(Component)]
struct Collide;
#[derive(Resource)]
struct UpdateRemainder(f32, bool);
#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Resource, Default)]
struct DebugInfo {
    text: Vec<TextSection>,
    collisions: Vec<(Aabb2d, (Vec2, Vec2))>,
    ctl_aabb: Option<Aabb2d>,
}
#[derive(Component)]
struct DebugUi;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(DebugInfo::default())
        .insert_resource(UpdateRemainder(0., false))
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
        .add_systems(Update, draw_debug)
        .run();
}

const MAP: [u8; 8 * 8] = [
    1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 0, 0, 0, 0, 1, 0, 1, // 2
    1, 0, 0, 0, 0, 0, 0, 1, // 3
    1, 0, 0, 0, 0, 1, 0, 1, // 4
    1, 0, 0, 0, 0, 0, 0, 1, // 5
    1, 0, 0, 0, 0, 0, 0, 1, // 6
    1, 0, 0, 0, 0, 0, 0, 1, // 7
    1, 1, 1, 1, 1, 1, 1, 1, // 8
];

fn setup_graphics(mut command: Commands, assets: Res<AssetServer>) {
    command.spawn(Camera2dBundle::default());
    command.spawn((
        DebugUi,
        Text2dBundle {
            text: Text::default(),
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform {
                translation: Vec3::new(-300., 300., 0.),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));
    command.spawn((
        Control,
        Collide,
        Velocity::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(0.8, 1.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                scale: Vec3::new(45., 45., 1.),
                ..default()
            },
            texture: assets.load("baby.png"),
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

    let v = Vec2::new(vx, vy);
    for mut c in &mut ctl {
        c.0 = v * 5.;
    }
}

// the intent is to cast the ctl's aabb along ctl's velocity and check for any collisions
// if there are any collisions, then reduce velocity until there aren't
//
// this is not working correctly as it sees collisions where it shouldn't
fn check_collide(
    time: Res<Time>,
    mut update_rem: ResMut<UpdateRemainder>,
    mut ctl: Query<(Entity, &Transform, &mut Velocity, &mut Sprite), With<Control>>,
    col: Query<(Entity, &Transform), With<Collide>>,
    mut dbg: ResMut<DebugInfo>,
) {
    for (e, t, mut v, mut s) in &mut ctl {
        if v.0 == Vec2::ZERO {
            continue;
        }

        let (mut dt, mut push_vert) = (update_rem.0, update_rem.1);
        dt += time.delta_seconds() * 60.;
        let mut aabb = Aabb2d::new(t.translation.xy(), t.scale.xy() / 2.);
        while dt > 1. {
            let mut collisions = vec![];
            aabb = Aabb2d::new(aabb.center() + v.xy(), aabb.half_size());
            for (ec, col) in &col {
                if ec == e {
                    continue;
                }

                let col_aabb = Aabb2d::new(col.translation.xy(), col.scale.xy() / 2.);
                if aabb.intersects(&col_aabb) {
                    collisions.push((col_aabb, (aabb.center(), col_aabb.center() - aabb.center())));
                    let left = col_aabb.min.x - aabb.max.x;
                    let right = col_aabb.max.x - aabb.min.x;
                    let up = col_aabb.max.y - aabb.min.y;
                    let down = col_aabb.min.y - aabb.max.y;
                    let horz = if left.abs() < right.abs() {
                        left
                    } else {
                        right
                    };
                    let vert = if up.abs() < down.abs() { up } else { down };
                    let push = match horz.abs().total_cmp(&vert.abs()) {
                        std::cmp::Ordering::Greater => Vec2::new(0., vert),
                        std::cmp::Ordering::Less => Vec2::new(horz, 0.),
                        std::cmp::Ordering::Equal => {
                            // stuck on a corner, if we always choose one or the other than there will be no progress
                            // so we choose vertical half the time and horizontal the other half
                            push_vert = !push_vert;
                            if push_vert {
                                Vec2::new(0., vert)
                            } else {
                                Vec2::new(horz, 0.)
                            }
                        }
                    };
                    aabb = Aabb2d::new(aabb.center() + push, aabb.half_size());
                }
            }

            if !collisions.is_empty() {
                s.color = Color::rgb(1., 0., 0.);

                dbg.text.clear();
                dbg.collisions = collisions;
                dbg.ctl_aabb = Some(Aabb2d::new(t.translation.xy() + **v, t.scale.xy() / 2.));
            } else {
                s.color = Color::rgb(0., 0., 1.);
            }

            dt -= 1.;
        }
        let tnew = aabb.center();
        **v = tnew - t.translation.xy();
        if (dt, push_vert) != (update_rem.0, update_rem.1) {
            update_rem.0 = dt;
            update_rem.1 = push_vert;
        }
    }
}

fn update_movement(mut movers: Query<(&mut Transform, &Velocity)>) {
    for (mut t, v) in &mut movers {
        t.translation.x += v.x;
        t.translation.y += v.y;
    }
}

fn draw_debug(dbg: Res<DebugInfo>, mut gizmos: Gizmos, mut ui: Query<&mut Text, With<DebugUi>>) {
    if dbg.is_changed() && !dbg.text.is_empty() {
        for mut ui in &mut ui {
            ui.sections = dbg.text.clone();
        }
    }
    if !dbg.collisions.is_empty() {
        for (aabb, (origin, ray)) in dbg.collisions.iter() {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::RED);
            gizmos.ray_2d(*origin, *ray, Color::GREEN);
        }
        if let Some(aabb) = &dbg.ctl_aabb {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::GREEN);
        }
    }
}
