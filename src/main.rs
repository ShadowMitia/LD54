use bevy::{prelude::*, sprite::collide_aabb::collide};

#[derive(Component)]
struct CollisionBox(Vec3);

#[derive(Component)]
struct TriggerBox(Vec3);

fn main() {
    App::new()
        .add_event::<OnGroundEvent>()
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
        .add_systems(Update, (trigger_ingredient_system, teller_system))
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

    // Floor
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::MAROON,
                custom_size: Some(Vec2::new(2000.0, 60.0)),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, -350.0, 0.0),
            ..default()
        },
        Collision,
        CollisionBox(Vec3::new(2000.0, 60.0, 0.0)),
    ));

    // Teller
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(24.0, 35.0)),
                ..default()
            },
            transform: Transform::from_xyz(-400.0, -300.0, 0.0),
            ..default()
        },
        Collision,
        CollisionBox(Vec3::new(24.0, 35.0, 0.0)),
        Teller,
        TriggerBox(Vec3::new(30.0, 36.0, 0.0)),
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
        CollisionBox(Vec3::new(32.0, 32.0, 0.0)),
        Inventory::new(),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::CYAN,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(-500.0, -200.0, 0.0),
            ..default()
        },
        NPC,
        Velocity(Vec3::ZERO),
        Acceleration(Vec3::ZERO),
        Collision,
        CollisionBox(Vec3::new(32.0, 32.0, 0.0)),
    ));

    spawn_ingredient(&mut commands, IngredientType::One);
    spawn_ingredient(&mut commands, IngredientType::Two);
    spawn_ingredient(&mut commands, IngredientType::Three);
    spawn_ingredient(&mut commands, IngredientType::Four);
}

fn gravity_system(mut q_physics: Query<&mut Acceleration>) {
    for mut acc in q_physics.iter_mut() {
        //info!("oh gravity");
        acc.0.y -= 1.0;
    }
}

fn physics_system(
    mut q_physics: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>,
    time_step: Res<FixedTime>,
) {
    for (mut trans, mut vel, mut acc) in q_physics.iter_mut() {
        //info!("updating physics");
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

#[derive(Event)]
struct OnGroundEvent {
    entity: Entity,
}

// NOTE: Assumes everything are rectangles
fn collision_system(
    q_colliders: Query<
        (&Transform, &CollisionBox),
        (With<Collision>, Without<Player>, Without<NPC>),
    >,
    mut q_player: Query<
        (
            Entity,
            &mut Transform,
            &Sprite,
            &mut Acceleration,
            &mut Velocity,
        ),
        (With<Collision>, Or<(With<Player>, With<NPC>)>),
    >,
    mut my_events: EventWriter<OnGroundEvent>,
) {
    // TODO: could be simplified with iter_combinations?
    for (ent, mut player_trans, player_sprite, mut acc, mut vel) in q_player.iter_mut() {
        for (transform, sprite) in q_colliders.iter() {
            //info!("checking collisions");
            let collision = collide(
                player_trans.translation,
                player_sprite.custom_size.unwrap(),
                transform.translation,
                sprite.0.truncate(),
            );

            if let Some(collision) = collision {
                //info!("collision");
                let half_size = sprite.0.truncate() / 2.0;
                let player_half_size = player_sprite.custom_size.unwrap() / 2.0;

                match collision {
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
                        my_events.send(OnGroundEvent { entity: ent });
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
}

#[derive(Component)]
struct Jumping;

// TODO: this still has double jump for some reason?
fn jump_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(Entity, &mut Acceleration, Option<&Jumping>), With<Player>>,
    mut events: EventReader<OnGroundEvent>,
) {
    for OnGroundEvent { entity } in events.iter() {
        commands.entity(*entity).remove::<Jumping>();
    }
    for (ent, mut acc, is_jumping) in player.iter_mut() {
        if !is_jumping.is_some() && keyboard_input.just_pressed(KeyCode::Space) {
            acc.0.y += 100.0;
            commands.entity(ent).insert(Jumping);
        }
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

#[derive(Component)]
struct Ingredient(IngredientType);

#[derive(Clone, PartialEq, Eq)]
enum IngredientType {
    One,
    Two,
    Three,
    Four,
}

fn spawn_ingredient(commands: &mut Commands, ingredient: IngredientType) {
    let color = {
        match ingredient {
            IngredientType::One => Color::YELLOW_GREEN,
            IngredientType::Two => Color::DARK_GREEN,
            IngredientType::Three => Color::LIME_GREEN,
            IngredientType::Four => Color::YELLOW,
        }
    };

    let position = {
        match ingredient {
            IngredientType::One => Vec3::new(0.0, -300.0, 0.0),
            IngredientType::Two => Vec3::new(200.0, -300.0, 0.0),
            IngredientType::Three => Vec3::new(-100.0, -300.0, 0.0),
            IngredientType::Four => Vec3::new(-80.0, -300.0, 0.0),
        }
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        Ingredient(ingredient),
        TriggerBox(Vec3::new(40.0, 40.0, 0.0)),
    ));
}

#[derive(Component)]
struct Teller;

fn trigger_ingredient_system(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &TriggerBox, &Ingredient), Without<Player>>,
    mut q_player: Query<(Entity, &Transform, &Sprite, &mut Inventory), With<Player>>,
) {
    let (player, player_trans, player_sprite, mut inventory) =
        q_player.get_single_mut().expect("Always a player");

    for (ingredient, mut transform, sprite, Ingredient(ing)) in q.iter_mut() {
        //info!("checking collisions");
        let collision = collide(
            player_trans.translation,
            player_sprite.custom_size.unwrap(),
            transform.translation,
            sprite.0.truncate(),
        );

        if collision.is_some() {
            if !inventory.items.contains(&Some(ing.clone())) {
                if inventory.items[0] == None {
                    inventory.items[0] = Some(ing.clone());
                } else if inventory.items[1] == None {
                    inventory.items[1] = Some(ing.clone());
                } else if inventory.items[2] == None {
                    inventory.items[2] = Some(ing.clone());
                } else {
                    // Inventory full
                    break;
                }

                let count = inventory.items.iter().filter(|&el| el != &None).count();

                let diff = match count {
                    0 => unreachable!(),
                    1 => -1.0,
                    2 => 0.0,
                    3 => 1.0,
                    _ => unreachable!(),
                };

                commands.entity(ingredient).set_parent(player);
                transform.scale = Vec3::new(0.5, 0.5, 0.0);
                transform.translation = Vec3::new(10.0 * diff, 10.0, 10.0);
            }
        }
    }
}

fn teller_system(
    mut commands: Commands,
    mut q: Query<(&mut Transform, &TriggerBox), (With<Teller>, Without<Player>)>,
    mut q_player: Query<
        (
            Entity,
            &Transform,
            &CollisionBox,
            &mut Inventory,
            Option<&Children>,
        ),
        With<Player>,
    >,
    q_ingredients: Query<(Entity, &Ingredient)>,
) {
    let (player, player_trans, player_sprite, mut inventory, children) =
        q_player.get_single_mut().expect("Always a player");

    for (transform, sprite) in q.iter_mut() {
        //info!("checking collisions");
        let collision = collide(
            player_trans.translation,
            player_sprite.0.truncate(),
            transform.translation,
            sprite.0.truncate(),
        );

        if collision.is_some() {
            if let Some(children) = children {
                info!("Checking {:?}", children);
                for &child in children.iter() {
                    if let Ok((_, Ingredient(ing))) = q_ingredients.get(child) {
                        if inventory.items[0] == Some(ing.clone()) {
                            inventory.items[0] = None;
                        } else if inventory.items[1] == Some(ing.clone()) {
                            inventory.items[1] = None;
                        } else if inventory.items[2] == Some(ing.clone()) {
                            inventory.items[2] = None;
                        }
                        commands.entity(player).despawn_descendants();
                        spawn_ingredient(&mut commands, ing.clone());
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct Inventory {
    items: [Option<IngredientType>; 3],
}

impl Inventory {
    fn new() -> Self {
        Self {
            items: [None, None, None],
        }
    }
}
