use bevy::{app::AppExit, prelude::*};
use rand::{thread_rng, Rng};
use std::ops::Add;

const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const SNEK_COLOR: Color = Color::rgb(91.0/255.0, 206.0/255.0, 250.0/255.0);
const APPLE_COLOR: Color = Color::rgb(245.0/255.0, 169.0/255.0, 184.0/255.0);

const SQUARE_SIZE: Vec2 = Vec2::new(60.0, 60.0);
const SNEK_PART_SIZE: Vec2 = Vec2::new(60.0, 60.0);
const APPLE_SIZE: Vec2 = Vec2::new(40.0, 40.0);

const GRID_SIZE: Vec2 = Vec2::new(14.0, 10.0);
const WINDOW_SIZE: Vec2 = Vec2::new(GRID_SIZE.x * SQUARE_SIZE.x, GRID_SIZE.y * SQUARE_SIZE.y);

const MAX_TIME_TO_MOVE: f32 = 0.3;
const MIN_TIME_TO_MOVE: f32 = 0.15;
const TIME_TO_REACH_MAX_SPEED: f32 = 60.0;

const SNAKE_BODY_PART_SIZE_MIN: f32 = 0.4;
const SNAKE_BODY_PART_SIZE_MAX: f32 = 0.9;
const MIN_SNAKE_LENGTH_SIZE_DIFF: usize = 5;

#[derive(PartialEq)]
enum SnekDirection {
    Left,
    Right,
    Up,
    Down
}

#[derive(PartialEq, Debug)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl Copy for Coordinate { }
impl Clone for Coordinate {
    fn clone(&self) -> Coordinate {
        *self
    }
}

impl Coordinate {
    fn to_screen_position(&self) -> Vec3 {
        return Vec3::new(
            (self.x as f32 - GRID_SIZE.x/2.0 + 0.5) * SQUARE_SIZE.x,
            (self.y as f32 - GRID_SIZE.y/2.0 + 0.5) * SQUARE_SIZE.y,
            0.0
        );
    }

    // method to make that takes x and y

    fn new(x: i32, y: i32) -> Coordinate {
        Coordinate { x, y }
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Component)]
struct Snek {
    position: Coordinate,
    length: usize,
    prev_positions: Vec<Coordinate>,
    time_since_last_move: f32,
    current_direction: SnekDirection
}

impl Snek {
    fn get_body_part_coordinate(&self, index: usize) -> &Coordinate {
        return &self.prev_positions[self.prev_positions.len()-1-index];
    }
}

#[derive(Component)]
struct SnekPart {
    index: usize
}

#[derive(Component)]
struct Apple {
    coordinate: Coordinate,
}

#[derive(Resource)]
struct GameData {
    first_apple_spawned: bool
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snek".into(),
                resolution: WINDOW_SIZE.into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(GameData { first_apple_spawned: false })
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                move_snek,
                update_snek_body_part_positions,
                add_first_apple
            )
            // `chain`ing systems together runs them in order
            .chain(),
        )
        .add_systems(Update, (update, bevy::window::close_on_esc))
        .run();
}

fn startup(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());

    let start_coordinate = Coordinate::new((GRID_SIZE.x/2.0) as i32, (GRID_SIZE.y/2.0) as i32);

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: SNEK_PART_SIZE.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: SNEK_COLOR,
                ..default()
            },
            ..default()
        },
        Snek {
            position: start_coordinate,
            length: 1,
            prev_positions: vec![start_coordinate],
            time_since_last_move: 0.0,
            current_direction: SnekDirection::Right
        },
        SnekPart {
            index: 0
        }
    ));
}

fn add_first_apple(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut query_snek: Query<&mut Snek>
) {
    // Workaround here because apparently commands.spawn doesn't actually spawn it yet
    // which makes it not query-able on Startup
    if !game_data.first_apple_spawned {
        game_data.first_apple_spawned = true;
        spawn_apple(&mut commands, &query_snek.single_mut());
    }
}

fn update(
    mut commands: Commands,
    query: Query<(Entity, &Apple)>,
    mut query_snek: Query<&mut Snek>
) {
    let mut snek = query_snek.single_mut();
    let snek_position = snek.position;
    
    // Going through each apple
    for (apple_entity, apple) in query.iter() {
        // If it is on the same position as the snek
        if apple.coordinate == snek_position {
            // Spawning a new apple to replace it
            spawn_apple(&mut commands, &snek);

            // Adding a new body part to the snake
            add_body_part(&mut commands, &snek);

            // Make the snek longer
            snek.length += 1;
        
            // And remove the apple
            commands.entity(apple_entity).despawn();
        }
    }
}

fn update_snek_body_part_positions(
    mut snek_part_query: Query<(&mut Transform, &mut SnekPart)>,
    mut query_snek: Query<&mut Snek>,
    mut exit: EventWriter<AppExit>
) {
    let snek = query_snek.single_mut();

    for (mut snek_part_transform, snek_part) in snek_part_query.iter_mut() {

        // Moving the body part along with the snake
        let coordinate = snek.get_body_part_coordinate(snek_part.index);
        snek_part_transform.translation = coordinate.to_screen_position();

        // Overlapping body parts
        for i in 0..snek.length {
            // Finding the index in the prev_positions vector
            let array_index = snek.prev_positions.len() - i - 1;
            
            // If it is not the same body part, but it is the same position, they must overlap
            if i != snek_part.index && snek.prev_positions[array_index] == *coordinate {
                // Game over
                exit.send(AppExit);
            }
        }

        // Calculating the size that this exact body part should be
        let pos_in_snake = snek_part.index as f32 / std::cmp::max(snek.length, MIN_SNAKE_LENGTH_SIZE_DIFF) as f32;
        let size = SNAKE_BODY_PART_SIZE_MAX - pos_in_snake * (SNAKE_BODY_PART_SIZE_MAX - SNAKE_BODY_PART_SIZE_MIN);
        let size = size * SNEK_PART_SIZE.extend(0.0);
        snek_part_transform.scale = size;
    }
}

fn move_snek(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_snek: Query<&mut Snek>,
    time: Res<Time>
) {
    // Finding the Snek and the transform of the snek
    let mut snek = query_snek.single_mut();

    // Taking input and changing the current direction accordingly
    if keyboard_input.pressed(KeyCode::Left) && snek.current_direction != SnekDirection::Right {
        snek.current_direction = SnekDirection::Left;
    }
    if keyboard_input.pressed(KeyCode::Right) && snek.current_direction != SnekDirection::Left {
        snek.current_direction = SnekDirection::Right;
    }
    if keyboard_input.pressed(KeyCode::Up) && snek.current_direction != SnekDirection::Down {
        snek.current_direction = SnekDirection::Up;
    }
    if keyboard_input.pressed(KeyCode::Down) && snek.current_direction != SnekDirection::Up {
        snek.current_direction = SnekDirection::Down;
    }

    // what the fuck is this min function, why no min (v1, v2) ??
    // Anyways this speeds up the snek over `TIME_TO_REACH_MAX_SPEED` seconds
    let curr_time_to_move = (1.0 as f32).min(time.elapsed_seconds() / TIME_TO_REACH_MAX_SPEED);
    let curr_time_to_move = MAX_TIME_TO_MOVE - curr_time_to_move * (MAX_TIME_TO_MOVE - MIN_TIME_TO_MOVE);

    // If enough time has passed since the last move, move again
    if snek.time_since_last_move > curr_time_to_move {

        // Finding an actual vector direction from the current direction
        let direction: Coordinate = match snek.current_direction {
            SnekDirection::Up => Coordinate::new(0, 1),
            SnekDirection::Down => Coordinate::new(0, -1),
            SnekDirection::Left => Coordinate::new(-1, 0),
            SnekDirection::Right => Coordinate::new(1, 0),
        };

        // Moving the snek
        snek.position = snek.position + direction;

        // If the new position is out of bounds, move to the other side (wrapping)
        snek.position.x = (snek.position.x + GRID_SIZE.x as i32) % GRID_SIZE.x as i32;
        snek.position.y = (snek.position.y + GRID_SIZE.y as i32) % GRID_SIZE.y as i32;

        // Saving the new position
        let new_position = snek.position;

        snek.prev_positions.push(new_position);

        // Reset move timer
        snek.time_since_last_move = 0.0;
    }

    snek.time_since_last_move += time.delta_seconds();
}

fn spawn_apple(commands: &mut Commands, snek: &Snek) {
    let coord_x = thread_rng().gen_range(0..GRID_SIZE.x as i32);
    let coord_y = thread_rng().gen_range(0..GRID_SIZE.y as i32);

    let mut coordinate = Coordinate::new(coord_x, coord_y);

    while snek.prev_positions.contains(&coordinate) {
        let coord_x = thread_rng().gen_range(0..GRID_SIZE.x as i32);
        let coord_y = thread_rng().gen_range(0..GRID_SIZE.y as i32);
    
        coordinate = Coordinate::new(coord_x, coord_y);
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: coordinate.to_screen_position(),
                scale: APPLE_SIZE.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: APPLE_COLOR,
                ..default()
            },
            ..default()
        },
        Apple {
            coordinate
        }
    ));
}

fn add_body_part(commands: &mut Commands, snek: &Snek) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: snek.get_body_part_coordinate(snek.length).to_screen_position(),
                scale: SNEK_PART_SIZE.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: SNEK_COLOR,
                ..default()
            },
            ..default()
        },
        SnekPart {
            index: snek.length
        }
    ));
}