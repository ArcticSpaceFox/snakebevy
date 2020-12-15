#![warn(clippy::complexity)]
use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::time::Duration;

const ARENA_HEIGHT: u32 = 20;
const ARENA_WIDTH: u32 = 20;

const FOOD_SPAWN_INTERVALL: u64 = 10000;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct SnakeHead {
    direction: Direction,
    try_direction: Direction,
}
struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

struct SnakeMoveTimer(Timer);

struct GameOverEvent;
struct GrowthEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

struct SnakeSegment;
#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_millis(FOOD_SPAWN_INTERVALL),
            true,
        ))
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.0, 1.0, 0.2).into()),
        segment_material: materials.add(Color::rgb(0.3, 0.5, 0.2).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    });
}

fn game_setup(mut commands: Commands, materials: Res<Materials>, segments: ResMut<SnakeSegments>) {
    commands
        .spawn(SpriteComponents {
            material: materials.food_material.clone(),
            ..Default::default()
        })
        .with(Food)
        .with(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .with(Size::square(0.8));
    spawn_initial_snake(commands, &materials, segments)
}

fn spawn_initial_snake(
    mut commands: Commands,
    materials: &Materials,
    mut segments: ResMut<SnakeSegments>,
) {
    let first_segment = spawn_segment(
        &mut commands,
        &materials.segment_material,
        Position { x: 3, y: 2 },
    );
    segments.0 = vec![first_segment];
    commands
        .spawn(SpriteComponents {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnakeHead {
            direction: Direction::Up,
            try_direction: Direction::Up,
        })
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8));
}

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            ..SpriteComponents::default()
        })
        .with(SnakeSegment)
        .with(position)
        .with(Size::square(0.65));
    commands.current_entity().unwrap()
}

fn handle_movement(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    for mut head in heads.iter_mut() {
        head.try_direction = if head.direction != Direction::Left
            && (keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A))
        {
            Direction::Left
        } else if head.direction != Direction::Down
            && (keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S))
        {
            Direction::Down
        } else if head.direction != Direction::Up
            && (keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W))
        {
            Direction::Up
        } else if head.direction != Direction::Right
            && (keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D))
        {
            Direction::Right
        } else {
            head.try_direction
        };
    }
}

fn snake_movement(
    snake_timer: ResMut<SnakeMoveTimer>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut last_tail_position: ResMut<LastTailPosition>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if !snake_timer.0.finished {
        return;
    }
    for (head_entity, mut head) in heads.iter_mut() {
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir = head.try_direction;
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        let last_head_pos = *head_pos;
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_events.send(GameOverEvent);
        }
        drop(head_pos);
        let mut segment_positions: Vec<Position> = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        if segment_positions.contains(&last_head_pos) {
            game_over_events.send(GameOverEvent);
        }
        segment_positions.insert(0, last_head_pos);
        segment_positions
            .iter()
            .zip(segments.0.iter())
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

#[allow(clippy::clippy::too_many_arguments)]
fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<Materials>,
    segments_res: ResMut<SnakeSegments>,
    segments: Query<(Entity, &SnakeSegment)>,
    food: Query<(Entity, &Food)>,
    heads: Query<(Entity, &SnakeHead)>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for (ent, _) in segments.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in food.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in heads.iter() {
            commands.despawn(ent);
        }
        spawn_initial_snake(commands, &materials, segments_res);
    }
}

fn snake_eating(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    head_positions: Query<With<SnakeHead, &Position>>,
) {
    if !snake_timer.0.finished {
        return;
    }
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    mut commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    materials: Res<Materials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        segments.0.push(spawn_segment(
            &mut commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ));
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in q.iter_mut() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / ARENA_WIDTH as f32 * window.width() as f32,
            size.height as f32 / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.) + (bound_window / bound_game / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    growth_events: Res<Events<GrowthEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished || growth_reader.iter(&growth_events).next().is_some() {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.8));
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 800,
            height: 800,
            ..Default::default()
        })
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis(150. as u64),
            true,
        )))
        .add_resource(SnakeSegments::default())
        .add_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(snake_timer.system())
        .add_system(handle_movement.system())
        .add_system(snake_movement.system())
        .add_system(snake_eating.system())
        .add_system(snake_growth.system())
        .add_system(food_spawner.system())
        .add_system(game_over.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_plugins(DefaultPlugins)
        .run();
}
