use std::time::Duration;

use bevy::{app::AppExit, prelude::*, sprite::collide_aabb::collide, utils::HashMap};

use rand::{distributions::Standard, prelude::Distribution, Rng};

#[derive(Component)]
struct CollisionBox(Vec3);

#[derive(Component)]
struct TriggerBox(Vec3);

#[derive(Resource)]
struct Score(usize);

#[derive(Resource)]
struct LevelTimer(Timer);

#[derive(Default, States, Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    MainMenu,
    #[default]
    InGame,
    EndScreen,
}

fn despawn_all<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    info!(
        "despawn all {:#?} ({})",
        std::any::type_name::<T>(),
        to_despawn.iter().count()
    );
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_event::<OnGroundEvent>()
        .insert_resource(Recipes(HashMap::new()))
        .insert_resource(Score(0))
        /* General systems */
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, bevy::window::close_on_esc))
        /* Main menu */
        .add_plugins(DefaultPlugins)
        .add_systems(OnEnter(GameState::MainMenu), setup_title_menu)
        .add_systems(
            Update,
            (title_menu, title_menu_action).run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), despawn_all::<TitleMenu>)
        /* In game systems */
        .add_systems(OnEnter(GameState::InGame), setup_game)
        .add_systems(
            Update,
            (
                movement_system,
                jump_system,
                gravity_system,
                physics_system,
                collision_system,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (
                level_timer_system,
                trigger_ingredient_system,
                teller_system,
                cooking_table_system,
                bin_system,
            )
                .run_if(in_state(GameState::InGame)),
        )
        /* End screen */
        .add_systems(OnEnter(GameState::EndScreen), setup_end_screen)
        .add_systems(Update, title_menu_action)
        .add_systems(
            OnExit(GameState::EndScreen),
            (despawn_all::<EndScreen>, despawn_all::<GameElement>),
        )
        /* Run all this mess! */
        .run();
}

#[derive(Component)]
struct NPC {
    wants: CakeType,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collision;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Acceleration(Vec3);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(AudioBundle {
        source: asset_server.load("music/Sakura Girl - Paris.ogg"),
        ..default()
    });
}

#[derive(Component)]
struct GameElement;

fn setup_game(
    mut commands: Commands,
    mut recipes: ResMut<Recipes>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
) {
    score.0 = 0;

    commands.insert_resource(LevelTimer(Timer::new(
        Duration::from_secs(60 * 15),
        TimerMode::Once,
    )));

    let r = vec![
        (
            CakeType::Chocolate,
            Recipe::new(&[
                IngredientType::Eggs,
                IngredientType::Flour,
                IngredientType::Chocolate,
                IngredientType::Milk,
            ]),
        ),
        (
            CakeType::Fraisier,
            Recipe::new(&[
                IngredientType::Eggs,
                IngredientType::Flour,
                IngredientType::Strawberry,
                IngredientType::Milk,
            ]),
        ),
        (
            CakeType::Carrot,
            Recipe::new(&[
                IngredientType::Eggs,
                IngredientType::Flour,
                IngredientType::Carrot,
                IngredientType::Milk,
            ]),
        ),
    ];

    for (cake, ings) in r {
        info!("Adding {:?} with {:?}", cake, ings);
        recipes.0.insert(ings, cake);
    }

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
        GameElement,
    ));

    // Teller
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/teller.png"),
            transform: Transform::from_xyz(-400.0, -300.0, 0.0)
                .with_scale(Vec3::new(0.3, 0.3, 0.0)),
            ..default()
        },
        Collision,
        CollisionBox(Vec3::new(24.0, 35.0, 0.0)),
        Teller,
        TriggerBox(Vec3::new(30.0, 36.0, 0.0)),
        GameElement,
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
        GameElement,
    ));

    let cake: CakeType = rand::random();

    let npc = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::CYAN,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(-500.0, -200.0, 0.0),
            ..default()
        },
        NPC {
            wants: cake.clone(),
        },
        Velocity(Vec3::ZERO),
        Acceleration(Vec3::ZERO),
        Collision,
        CollisionBox(Vec3::new(32.0, 32.0, 0.0)),
        GameElement,
    ));

    let id = &npc.id();

    spawn_display_cake(
        &asset_server,
        &mut commands,
        Vec3::new(0.0, 30.0, 0.0),
        cake,
        id,
    );

    spawn_ingredient(&asset_server, &mut commands, IngredientType::Eggs);
    spawn_ingredient(&asset_server, &mut commands, IngredientType::Flour);
    spawn_ingredient(&asset_server, &mut commands, IngredientType::Chocolate);
    spawn_ingredient(&asset_server, &mut commands, IngredientType::Milk);
    spawn_ingredient(&asset_server, &mut commands, IngredientType::Strawberry);
    spawn_ingredient(&asset_server, &mut commands, IngredientType::Carrot);

    // Cooking table
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/cake_machine.png"),
            transform: Transform::from_xyz(400.0, -300.0, 0.0).with_scale(Vec3::new(0.3, 0.3, 0.0)),
            ..default()
        },
        Collision,
        CollisionBox(Vec3::new(44.0, 35.0, 0.0)),
        CookingTable,
        TriggerBox(Vec3::new(50.0, 40.0, 0.0)),
        GameElement,
    ));

    // Bin
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/bin.png"),
            transform: Transform::from_xyz(-350.0, -300.0, 0.0)
                .with_scale(Vec3::new(0.3, 0.3, 0.0)),
            ..default()
        },
        Bin,
        TriggerBox(Vec3::new(50.0, 40.0, 0.0)),
        GameElement,
    ));
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
        if is_jumping.is_none() && keyboard_input.just_pressed(KeyCode::Space) {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IngredientType {
    Eggs,
    Flour,
    Chocolate,
    Milk,
    Strawberry,
    Carrot,
}

fn spawn_ingredient(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    ingredient: IngredientType,
) {
    let color = {
        match ingredient {
            IngredientType::Eggs => asset_server.load("sprites/eggs.png"),
            IngredientType::Flour => asset_server.load("sprites/flour.png"),
            IngredientType::Chocolate => asset_server.load("sprites/chocolate.png"),
            IngredientType::Milk => asset_server.load("sprites/milk.png"),
            IngredientType::Strawberry => asset_server.load("sprites/strawberry.png"),
            IngredientType::Carrot => asset_server.load("sprites/carrot.png"),
        }
    };

    let position = {
        match ingredient {
            IngredientType::Eggs => Vec3::new(0.0, -300.0, 0.0),
            IngredientType::Flour => Vec3::new(200.0, -300.0, 0.0),
            IngredientType::Chocolate => Vec3::new(-100.0, -300.0, 0.0),
            IngredientType::Milk => Vec3::new(-80.0, -300.0, 0.0),
            IngredientType::Strawberry => Vec3::new(250.0, -300.0, 0.0),
            IngredientType::Carrot => Vec3::new(150.0, -300.0, 0.0),
        }
    };

    commands.spawn((
        SpriteBundle {
            texture: color,
            transform: Transform::from_translation(position).with_scale(Vec3::new(0.4, 0.4, 0.0)),
            ..default()
        },
        Ingredient(ingredient),
        TriggerBox(Vec3::new(40.0, 40.0, 0.0)),
        GameElement,
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

        if collision.is_some() && !inventory.items.contains(&Some(ing.clone())) {
            let toto = inventory.items.iter_mut().find(|el| el.is_none());

            if let Some(toto) = toto {
                *toto = Some(ing.clone());
            } else {
                // Inventory full
                break;
            }

            let count = inventory.items.iter().filter(|&el| el.is_some()).count();

            let diff = match count {
                0 => unreachable!(),
                1 => -1.5,
                2 => -0.75,
                3 => 0.75,
                4 => 1.5,
                _ => unreachable!(),
            };

            commands.entity(ingredient).set_parent(player);
            transform.scale = Vec3::new(0.3, 0.3, 0.0);
            transform.translation = Vec3::new(10.0 * diff, 10.0, 10.0);
        }
    }
}

fn teller_system(
    mut commands: Commands,
    mut q: Query<(&mut Transform, &TriggerBox), (With<Teller>, Without<Player>)>,
    mut q_npc: Query<
        (Entity, &mut Transform, &mut NPC),
        (Without<Player>, With<NPC>, Without<Teller>),
    >,
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
    q_ingredients: Query<(Entity, &Cake)>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
) {
    let (_player, player_trans, player_sprite, mut inventory, _children) =
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
            if let Some(_cake) = &inventory.cake {
                let (npc_e, mut npc_trans, mut npc) =
                    q_npc.get_single_mut().expect("Always an NPC");

                let (ent, Cake(cake)) = q_ingredients.get_single().expect("Should be a cake there");
                if npc.wants == *cake {
                    score.0 += 1;

                    commands.entity(ent).despawn_recursive();
                    commands.entity(npc_e).despawn_descendants();

                    inventory.cake = None;
                    npc_trans.translation.y += 50.0;
                    npc.wants = rand::random();

                    spawn_display_cake(
                        &asset_server,
                        &mut commands,
                        Vec3::new(0.0, 30.0, 0.0),
                        npc.wants.clone(),
                        &npc_e,
                    );
                }
            }
        }
    }
}

#[derive(Component)]
struct Inventory {
    items: [Option<IngredientType>; 4],
    cake: Option<CakeType>,
}

impl Inventory {
    fn new() -> Self {
        Self {
            items: [None, None, None, None],
            cake: None,
        }
    }
}

#[derive(Component)]
struct CookingTable;

#[derive(Component)]
struct Cake(CakeType);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CakeType {
    Chocolate,
    Fraisier,
    Carrot,
}

// NOTE: Could be a macro to autogen?
impl Distribution<CakeType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CakeType {
        let index: u8 = rng.gen_range(0..3);
        match index {
            0 => CakeType::Chocolate,
            1 => CakeType::Fraisier,
            2 => CakeType::Carrot,
            _ => unreachable!(),
        }
    }
}

fn spawn_cake(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    position: Vec3,
    cake: CakeType,
    parent: &Entity,
) -> Entity {
    let color = {
        match cake {
            CakeType::Chocolate => asset_server.load("sprites/cake_chocolate.png"),
            CakeType::Fraisier => asset_server.load("sprites/cake_strawberry.png"),
            CakeType::Carrot => asset_server.load("sprites/cake_carrot.png"),
        }
    };

    let id = commands
        .spawn((
            SpriteBundle {
                texture: color,
                transform: Transform::from_translation(position),
                ..default()
            },
            Cake(cake),
            GameElement,
        ))
        .set_parent(*parent)
        .id();
    id
}

fn spawn_display_cake(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    position: Vec3,
    cake: CakeType,
    parent: &Entity,
) -> Entity {
    let color = {
        match cake {
            CakeType::Chocolate => asset_server.load("sprites/cake_chocolate.png"),
            CakeType::Fraisier => asset_server.load("sprites/cake_strawberry.png"),
            CakeType::Carrot => asset_server.load("sprites/cake_carrot.png"),
        }
    };

    let id = commands
        .spawn((
            SpriteBundle {
                texture: color,
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::new(0.4, 0.4, 0.0)),
                ..default()
            },
            GameElement,
        ))
        .set_parent(*parent)
        .id();
    id
}

fn cooking_table_system(
    mut commands: Commands,
    mut q: Query<(&mut Transform, &TriggerBox), (With<CookingTable>, Without<Player>)>,
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
    _q_ingredients: Query<(Entity, &Ingredient)>,
    recipes: Res<Recipes>,
    asset_server: Res<AssetServer>,
) {
    let (player, player_trans, player_sprite, mut inventory, _children) =
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
            let cake = 'cake: {
                'recipes: for (Recipe { ingredients }, v) in recipes.0.iter() {
                    for ing in inventory.items.iter() {
                        if let Some(ing) = ing {
                            if !ingredients.contains(ing) {
                                // Check next recipe
                                continue 'recipes;
                            }
                        } else {
                            // Got None, missing ingredients
                            return;
                        }
                    }
                    break 'cake Some(v.clone());
                }
                None
            };

            if let Some(cake) = cake {
                // We have cake!

                // Clear all ingredients
                for item in inventory.items.iter_mut() {
                    if let Some(item) = item {
                        spawn_ingredient(&asset_server, &mut commands, item.clone());
                    }
                    *item = None;
                }
                // Clear what player is carrying
                commands.entity(player).despawn_descendants();

                // Spawn the cake
                spawn_cake(
                    &asset_server,
                    &mut commands,
                    Vec3::new(0.0, 40.0, 0.0),
                    cake.clone(),
                    &player,
                );

                // Add to inventory
                inventory.cake = Some(cake);
            }
        }
    }
}

#[derive(Resource)]
struct Recipes(HashMap<Recipe, CakeType>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Recipe {
    ingredients: Vec<IngredientType>,
}

impl Recipe {
    fn new(ingredients: &[IngredientType]) -> Self {
        Self {
            ingredients: Vec::from(ingredients),
        }
    }
}

#[derive(Component)]
struct Bin;

fn bin_system(
    mut commands: Commands,
    q_bin: Query<(&Transform, &TriggerBox), With<Bin>>,
    mut q_player: Query<(Entity, &Transform, &CollisionBox, &mut Inventory)>,
    keyboard_input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let (player, player_trans, player_box, mut inventory) =
        q_player.get_single_mut().expect("Always a player");

    let (bin_trans, bin_box) = q_bin.get_single().expect("Always a bin");

    //info!("checking collisions");
    let collision = collide(
        player_trans.translation,
        player_box.0.truncate(),
        bin_trans.translation,
        bin_box.0.truncate(),
    );

    if collision.is_some() && keyboard_input.just_pressed(KeyCode::C) {
        for item in inventory.items.iter_mut() {
            if let Some(ing) = item {
                spawn_ingredient(&asset_server, &mut commands, ing.clone());
            }
            *item = None;
        }
        inventory.cake = None;

        commands.entity(player).despawn_descendants();
    }
}

/// Spawn a new bomb in set intervals of time
fn level_timer_system(
    time: Res<Time>,
    mut timer: ResMut<LevelTimer>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    // tick the timer
    timer.0.tick(time.delta());

    if timer.0.finished() {
        app_state.set(GameState::EndScreen);
    }
}

// MAIN MENU

#[derive(Component)]
struct TitleMenu;

#[derive(Component)]
enum TitleMenuAction {
    NewGame,
    Quit,
}

fn setup_title_menu(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font_size: 40.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            TitleMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
			position_type: PositionType::Absolute,
			bottom: Val::Px(0.0),
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(r#"    Music "Paris" by Sakura Girl (https://officialsakuragirl.wixsite.com/official)"#,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                        .with_style(Style { ..default() }),
                        // Because this is a distinct label widget and
                        // not button/list item text, this is necessary
                        // for accessibility to treat the text accordingly.
                        Label,
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Truly Teeny Tiny Bakery",
                            TextStyle {
                                font_size: 60.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        }),
                        // Because this is a distinct label widget and
                        // not button/list item text, this is necessary
                        // for accessibility to treat the text accordingly.
                        Label,
                    ));

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TitleMenuAction::NewGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "New Game",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TitleMenuAction::Quit,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Quit", button_text_style.clone()));
                        });
                });
        });
}

fn title_menu(keyboard_input: Res<Input<KeyCode>>, mut app_state: ResMut<NextState<GameState>>) {}

fn title_menu_action(
    interaction_query: Query<
        (&Interaction, &TitleMenuAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                TitleMenuAction::NewGame => app_state.set(GameState::InGame),
                TitleMenuAction::Quit => app_exit_events.send(AppExit),
            }
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

#[derive(Component)]
struct EndScreen;

fn setup_end_screen(mut commands: Commands, score: Res<Score>) {
    let score = score.0;

    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Percent(100.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font_size: 40.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            EndScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            format!("Well done!\nYou sold {score} cakes!"),
                            TextStyle {
                                font_size: 60.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        }),
                        // Because this is a distinct label widget and
                        // not button/list item text, this is necessary
                        // for accessibility to treat the text accordingly.
                        Label,
                    ));

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TitleMenuAction::NewGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "New Game",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TitleMenuAction::Quit,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Quit", button_text_style.clone()));
                        });
                });
        });
}
