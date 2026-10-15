use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::body::Appearence;
use crate::health::{Health, MELEE_DAMAGE};
use crate::database::Database;
use crate::objects::{self, Solid, Tree, TreeSounds};
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::utils::{self, TILE_SIZE};

pub const MOVEMENT_SPEED: f32 = 3.0;

pub struct Position {
    pub value : Vec2,
    pub offset : Vec2
}

pub struct BezierMovement{
    pub p0 : Vec2,
    pub p1 : Vec2,
    pub p2 : Vec2,
    pub t : f32,
    pub has_started :bool,
    pub speed : f32
}

pub struct AttackAction {
    pub origin: Vec2,
    pub target: Vec2,
    pub target_entity: Option<Entity>,
    pub path: Vec<Vec2>,
    pub returning: bool,
    pub striking: bool,
    pub active: bool,
}

pub struct RandomWalker {
}

pub fn start_attack(world: &World, target_entity: Entity, target: Vec2) -> bool {
    let target = (target / TILE_SIZE).round() * TILE_SIZE;
    let Some((player_position, player_moving)) = world
        .query::<(&Player, &Position, &BezierMovement)>()
        .iter()
        .next()
        .map(|(_, position, movement)| (position.value, movement.has_started))
    else {
        return false;
    };
    if player_moving {
        return false;
    }

    let Some(path) = path_to_attack_position(world, player_position, target) else {
        return false;
    };
    let attack_position = path.last().copied().unwrap_or(player_position);

    for (position, bezier, attack, _) in world
        .query::<(&Position, &mut BezierMovement, &mut AttackAction, &Player)>()
        .iter()
    {
        if attack.active || bezier.has_started {
            return false;
        }

        attack.origin = attack_position;
        attack.target = target;
        attack.target_entity = Some(target_entity);
        attack.path = path;
        attack.returning = false;
        attack.striking = false;
        attack.active = true;

        if attack.path.is_empty() {
            start_attack_strike(bezier, position.value, attack);
        } else {
            start_movement(bezier, position.value, attack.path.remove(0));
        }
        return true;
    }

    false
}

pub fn advance_attack_actions(
    world: &mut World,
    database: &Database,
    sounds: Option<&TreeSounds>,
) {
    let mut targets_hit = Vec::new();
    {
        for (position, bezier, attack, _) in world
            .query::<(&Position, &mut BezierMovement, &mut AttackAction, &Player)>()
            .iter()
        {
            if !attack.active || bezier.has_started {
                continue;
            }

            if attack.returning {
                attack.active = false;
                attack.target_entity = None;
                continue;
            }

            let position = position.value;

            if attack.striking {
                if let Some(target) = attack.target_entity {
                    targets_hit.push((target, attack.origin));
                }
                attack.striking = false;
                attack.returning = true;
                start_movement(bezier, position, attack.origin);
            } else if let Some(destination) = attack.path.first().copied() {
                attack.path.remove(0);
                start_movement(bezier, position, destination);
            } else {
                start_attack_strike(bezier, position, attack);
            }
        }
    }

    for (target, player_position) in targets_hit {
        if world.get::<&Tree>(target).is_ok() {
            objects::finish_tree_felling(world, database, sounds, target, player_position);
        } else {
            let defeated = world
                .get::<&mut Health>(target)
                .is_ok_and(|mut health| health.take_damage(MELEE_DAMAGE));
            if defeated {
                world
                    .despawn(target)
                    .expect("defeated target disappeared before despawning");
            }
        }
    }
}

fn path_to_attack_position(world: &World, start: Vec2, target: Vec2) -> Option<Vec<Vec2>> {
    let start_cell = (
        (start.x / TILE_SIZE).round() as i32,
        (start.y / TILE_SIZE).round() as i32,
    );
    let target_cell = (
        (target.x / TILE_SIZE).round() as i32,
        (target.y / TILE_SIZE).round() as i32,
    );
    let blocked_cells: HashSet<(i32, i32)> = world
        .query::<(&Position, &Solid)>()
        .iter()
        .map(|(position, _)| cell_coordinates(position.value))
        .chain(
            world
                .query::<(&Position, &RandomWalker)>()
                .iter()
                .map(|(position, _)| cell_coordinates(position.value)),
        )
        .collect();

    let mut parents = HashMap::from([(start_cell, None)]);
    let mut queue = VecDeque::from([start_cell]);
    let directions = [(0, -1), (-1, 0), (1, 0), (0, 1)];
    let mut destination = None;

    while let Some(cell) = queue.pop_front() {
        if is_attack_position(cell, target_cell) {
            destination = Some(cell);
            break;
        }

        for (dx, dy) in directions {
            let next = (cell.0 + dx, cell.1 + dy);
            if next.0 < 0
                || next.1 < 0
                || next.0 >= MAP_WIDTH as i32
                || next.1 >= MAP_HEIGHT as i32
                || blocked_cells.contains(&next)
                || parents.contains_key(&next)
            {
                continue;
            }

            parents.insert(next, Some(cell));
            queue.push_back(next);
        }
    }

    let mut cell = destination?;
    let mut path = Vec::new();
    while cell != start_cell {
        path.push(vec2(cell.0 as f32, cell.1 as f32) * TILE_SIZE);
        cell = parents[&cell].expect("attack path cell must have a parent");
    }
    path.reverse();
    Some(path)
}

fn cell_coordinates(position: Vec2) -> (i32, i32) {
    (
        (position.x / TILE_SIZE).round() as i32,
        (position.y / TILE_SIZE).round() as i32,
    )
}

fn is_attack_position(cell: (i32, i32), target: (i32, i32)) -> bool {
    (cell.0 - target.0).abs() + (cell.1 - target.1).abs() == 1
}

fn start_attack_strike(bezier: &mut BezierMovement, position: Vec2, attack: &mut AttackAction) {
    attack.striking = true;
    start_movement(bezier, position, attack.target);
}

fn start_movement(bezier: &mut BezierMovement, from: Vec2, to: Vec2) {
    bezier.p0 = from;
    bezier.p1 = (from + to) * 0.5 + vec2(0.0, -16.0);
    bezier.p2 = to;
    bezier.t = 0.0;
    bezier.has_started = true;
}

pub fn random_walking(world: &World) {
    let blocked_positions: Vec<Vec2> = world
        .query::<(&Position, &Solid)>()
        .iter()
        .map(|(position, _)| position.value)
        .collect();
    let mut occupied_positions: Vec<Vec2> = world
        .query::<(&Position, &RandomWalker)>()
        .iter()
        .map(|(position, _)| (position.value / TILE_SIZE).round() * TILE_SIZE)
        .collect();
    for (position, movement, _) in world
        .query::<(&Position, &BezierMovement, &Player)>()
        .iter()
    {
        occupied_positions.push((position.value / TILE_SIZE).round() * TILE_SIZE);
        if movement.has_started {
            occupied_positions.push(movement.p2);
        }
    }
    for (attack, _) in world.query::<(&AttackAction, &Player)>().iter() {
        occupied_positions.extend(attack.path.iter().copied());
    }

    let excluded_target = world
        .query::<(&AttackAction, &Player)>()
        .iter()
        .find_map(|(attack, _)| attack.active.then_some(attack.target_entity).flatten());
    for (entity, _, bezier, pos) in world
        .query::<(Entity, &mut RandomWalker, &mut BezierMovement, &Position)>()
        .iter()
    {
        if Some(entity) == excluded_target {
            continue;
        }

        let current_cell = pos.value / TILE_SIZE;
        let options: Vec<Vec2> = [
            vec2(-1.0, 0.0),
            vec2(1.0, 0.0),
            vec2(0.0, -1.0),
            vec2(0.0, 1.0),
        ]
        .into_iter()
        .map(|direction| current_cell + direction)
        .filter(|cell| {
            cell.x >= 0.0
                && cell.y >= 0.0
                && cell.x < MAP_WIDTH as f32
                && cell.y < MAP_HEIGHT as f32
                && !blocked_positions.contains(&(*cell * TILE_SIZE))
                && !occupied_positions.contains(&(*cell * TILE_SIZE))
        })
        .collect();

        if options.is_empty() {
            continue;
        }

        let destination = options[macroquad::rand::gen_range(0, options.len() as i32) as usize]
            * TILE_SIZE;
        occupied_positions.push(destination);

        bezier.p0 = pos.value;
        bezier.p1 = (pos.value + destination) * 0.5 + vec2(0.0, -16.0);
        bezier.p2 = destination;
        bezier.t = 0.0;
        bezier.has_started = true;
    }
}

pub fn entities_are_moving(world: &World) -> bool {
    world
        .query::<&BezierMovement>()
        .iter()
        .any(|movement| movement.has_started)
}


pub fn bezier_movement(world: &World, dt : f32) {
    for ( bezier, pos) in world.query::<(&mut BezierMovement, &mut Position)>().iter() {
        if bezier.has_started == false{
            continue;
        }

        bezier.t += dt * bezier.speed;

        let movement = utils::bezier(bezier.p0, bezier.p1, bezier.p2, bezier.t);
        pos.value = movement;


        if bezier.t >= 1.0{
            bezier.has_started = false;
            bezier.t = 0.0;

            pos.value = bezier.p2;
        }
    }
}

pub fn camera_follow(world : &World,camera : &mut Camera2D){

    for (_, position) in world.query::<(&Player, &Position)>().iter(){
        camera.target = position.value;
    }
}

pub fn flip_system(world: &World) {
    for (body, bezier) in world
        .query::<(&mut Appearence, &BezierMovement)>()
        .iter()
    {
        let dir = bezier.p2.x - bezier.p0.x;

        if dir < 0.0 {
            body.is_flipped = true;
        } else if dir > 0.0 {
            body.is_flipped = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::MAX_HEALTH;

    #[test]
    fn distant_attack_walks_to_target_then_strikes_and_returns_to_melee_range() {
        let database = Database::new();
        let mut world = World::new();
        let origin = vec2(32.0, 64.0);
        let target = vec2(96.0, 64.0);
        let target_entity = world.spawn((Position {
            value: target,
            offset: Vec2::ZERO,
        }, BezierMovement {
            p0: target,
            p1: target,
            p2: target,
            t: 0.0,
            has_started: false,
            speed: MOVEMENT_SPEED,
        }, RandomWalker {}, Health::new(MAX_HEALTH)));
        let player = world.spawn((
            Player {},
            Position {
                value: origin,
                offset: Vec2::ZERO,
            },
            BezierMovement {
                p0: origin,
                p1: origin,
                p2: origin,
                t: 0.0,
                has_started: false,
                speed: MOVEMENT_SPEED,
            },
            AttackAction {
                origin: Vec2::ZERO,
                target: Vec2::ZERO,
                target_entity: None,
                path: Vec::new(),
                returning: false,
                striking: false,
                active: false,
            },
        ));

        assert!(start_attack(&world, target_entity, target));
        random_walking(&world);
        assert!(!world
            .get::<&BezierMovement>(target_entity)
            .unwrap()
            .has_started);
        {
            let movement = world.get::<&BezierMovement>(player).unwrap();
            let action = world.get::<&AttackAction>(player).unwrap();
            assert_eq!(movement.p2, vec2(64.0, 64.0));
            assert!(action.active);
            assert!(!action.returning);
            assert_eq!(action.target_entity, Some(target_entity));
        }

        move_player_to_destination(&world, player);
        advance_attack_actions(&mut world, &database, None);
        {
            let movement = world.get::<&BezierMovement>(player).unwrap();
            let action = world.get::<&AttackAction>(player).unwrap();
            assert_eq!(movement.p2, target);
            assert!(movement.has_started);
            assert!(action.striking);
        }

        move_player_to_destination(&world, player);
        advance_attack_actions(&mut world, &database, None);
        assert_eq!(
            world.get::<&Health>(target_entity).unwrap().current,
            MAX_HEALTH - MELEE_DAMAGE
        );
        {
            let movement = world.get::<&BezierMovement>(player).unwrap();
            let action = world.get::<&AttackAction>(player).unwrap();
            assert_eq!(movement.p2, vec2(64.0, 64.0));
            assert!(movement.has_started);
            assert!(action.active);
            assert!(action.returning);
        }

        move_player_to_destination(&world, player);
        advance_attack_actions(&mut world, &database, None);
        let action = world.get::<&AttackAction>(player).unwrap();
        assert!(!action.active);
        assert_eq!(action.target_entity, None);
        assert_eq!(
            world.get::<&Position>(player).unwrap().value,
            vec2(64.0, 64.0)
        );
        assert_eq!(world.get::<&Position>(target_entity).unwrap().value, target);
    }

    fn move_player_to_destination(world: &World, player: Entity) {
        let destination = {
            let movement = world.get::<&BezierMovement>(player).unwrap();
            movement.p2
        };
        world.get::<&mut Position>(player).unwrap().value = destination;
        world.get::<&mut BezierMovement>(player).unwrap().has_started = false;
    }
}