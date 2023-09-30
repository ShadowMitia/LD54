use bevy::{prelude::*, sprite::collide_aabb::collide};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                movement_system,
                jump_system,
                gravity_system,
                physics_system,
                collision_system,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct NPC;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collision;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Acceleration(Vec3);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::MAROON,
                custom_size: Some(Vec2::new(500.0, 60.0)),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, -60.0, 0.0),
            ..default()
        },
        Collision,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(64.0, 60.0)),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, 0.0, 0.0),
            ..default()
        },
        Collision,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            ..default()
        },
        Player,
        Velocity(Vec3::ZERO),
        Acceleration(Vec3::ZERO),
        Collision,
    ));
}

fn gravity_system(mut q_physics: Query<&mut Acceleration>) {
    for mut acc in q_physics.iter_mut() {
        info!("oh gravity");
        acc.0.y -= 1.0;
    }
}

fn physics_system(
    mut q_physics: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>,
    time_step: Res<FixedTime>,
) {
    for (mut trans, mut vel, mut acc) in q_physics.iter_mut() {
        info!("updating physics");
        vel.0 += acc.0;
        acc.0 = Vec3::ZERO;
        /*
                if vel.0.length() > 20.0 {
                    vel.0 = vel.0.normalize_or_zero() * 20.0;
                }
        */
        trans.translation += vel.0 * time_step.period.as_secs_f32();
    }
}

fn collision_system(
    q_colliders: Query<(&Transform, &Sprite), (With<Collision>, Without<Player>)>,
    mut q_player: Query<(&mut Transform, &Sprite, &mut Acceleration, &mut Velocity), With<Player>>,
) {
    let (mut player_trans, player_sprite, mut acc, mut vel) =
        q_player.get_single_mut().expect("Always a player");

    for (transform, sprite) in q_colliders.iter() {
        info!("checking collisions");
        let collision = collide(
            player_trans.translation,
            player_sprite.custom_size.unwrap(),
            transform.translation,
            sprite.custom_size.unwrap(),
        );

        if let Some(collision) = collision {
            info!("collision");
            let half_size = sprite.custom_size.unwrap() / 2.0;
            let player_half_size = player_sprite.custom_size.unwrap() / 2.0;

            match dbg!(collision) {
                bevy::sprite::collide_aabb::Collision::Left => {
                    acc.0.x = 0.0;
                    vel.0.x = 0.0;
                    player_trans.translation.x =
                        transform.translation.x - half_size.x - player_half_size.x
                }
                bevy::sprite::collide_aabb::Collision::Right => {
                    acc.0.x = 0.0;
                    vel.0.x = 0.0;
                    player_trans.translation.x =
                        transform.translation.x + half_size.x + player_half_size.x
                }
                bevy::sprite::collide_aabb::Collision::Top => {
                    acc.0.y = 0.0;
                    vel.0.y = 0.0;
                    player_trans.translation.y =
                        transform.translation.y + half_size.y + player_half_size.y
                }
                bevy::sprite::collide_aabb::Collision::Bottom => {
                    acc.0.y = 0.0;
                    vel.0.y = 0.0;
                    player_trans.translation.y =
                        transform.translation.y - half_size.y - player_half_size.y
                }
                bevy::sprite::collide_aabb::Collision::Inside => unreachable!(),
            }
        }
    }
}

fn jump_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    let mut player = player.get_single_mut().expect("Always a player");

    if keyboard_input.just_pressed(KeyCode::Space) {
        player.0.y += 100.0;
    }
}

fn movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut Transform, With<Player>>,
    time_step: Res<FixedTime>,
) {
    let mut player = player.get_single_mut().expect("Always a player");

    if keyboard_input.pressed(KeyCode::Left) {
        player.translation.x -= 100.0 * time_step.period.as_secs_f32();
    } else if keyboard_input.pressed(KeyCode::Right) {
        player.translation.x += 100.0 * time_step.period.as_secs_f32();
    }
}
