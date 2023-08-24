#![allow(clippy::type_complexity)]

use valence::prelude::*;
use valence_command::arg_parser::{CommandArg, EntitySelector, EntitySelectors, GreedyString, QuotableString};

use valence_command::command_scopes::CommandScopes;
use valence_command::handler::{CommandResultEvent, CommandHandler};

use valence_command::{arg_parser, CommandScopeRegistry};
use valence_command_derive::Command;

const SPAWN_Y: i32 = 64;

#[derive(Command, Debug, Clone)]
#[paths("teleport", "tp")]
#[scopes("valence:command:teleport")]
enum Teleport {
    #[paths = "{location}"]
    ExecutorToLocation { location: arg_parser::Vec3 },
    #[paths = "{target}"]
    ExecutorToTarget { target: EntitySelector },
    #[paths = "{from} {to}"]
    TargetToTarget { from: EntitySelector, to: EntitySelector },
    #[paths = "{target} {location}"]
    TargetToLocation {
        target: EntitySelector,
        location: arg_parser::Vec3,
    },
}

#[derive(Command, Debug, Clone)]
#[paths("test", "t")]
#[scopes("valence:command:teleport")]
enum Test {
    // 3 literals with an arg each
    #[paths("a {a} b {b} c {c}", "{a} {b} {c}")]
    A { a: String, b: i32, c: f32 },
    // 2 literals with an arg last being optional (Because of the greedy string before the end this
    // is technically unreachable)
    #[paths = "a {a} {b} b {c?}"]
    B {
        a: String,
        b: GreedyString,
        c: Option<String>,
    },
    // greedy string optional arg
    #[paths = "a {a} b {b?}"]
    C { a: String, b: Option<GreedyString> },
    // greedy string required arg
    #[paths = "a {a} b {b}"]
    D { a: String, b: GreedyString },
    // five optional args and an ending greedyString
    #[paths("options {a?} {b?} {c?} {d?} {e?}", "options {b?} {a?} {d?} {c?} {e?}")]
    E {
        a: Option<i32>,
        b: Option<QuotableString>,
        c: Option<arg_parser::Vec2>,
        d: Option<arg_parser::Vec3>,
        e: Option<GreedyString>,
    },
}



pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CommandHandler::<Teleport>::from_command(),
            CommandHandler::<Test>::from_command(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                despawn_disconnected_clients,
                toggle_perms_on_sneak,
                handle_teleport_command,
            ),
        )
        .run();
}

fn handle_teleport_command(
    mut events: EventReader<CommandResultEvent<Teleport>>,
    mut clients: Query<(&mut Client, &mut Position)>,
    usernames: Query<(Entity, &Username)>, // mut commands: Commands
) {
    for event in events.iter() {
        match &event.result {
            Teleport::ExecutorToLocation { location } => {
                let (client, pos) = &mut clients.get_mut(event.executor).unwrap();
                pos.0.x = location.x.get(pos.0.x as f32) as f64;
                pos.0.y = location.y.get(pos.0.y as f32) as f64;
                pos.0.z = location.z.get(pos.0.z as f32) as f64;

                client.send_chat_message(format!(
                    "Teleport command executor -> location executed with data:\n {:#?}",
                    &event.result
                ));
            }
            Teleport::ExecutorToTarget { target } => {
                let raw_target = match target {
                    EntitySelector::SimpleSelector(x) => match
                    x {
                        EntitySelectors::SinglePlayer(x) => x,
                        _ => "not implemented",
                    },
                    _ => "not implemented",
                };
                let target = usernames.iter().find(|(_, name)| name.0 == *raw_target);

                match target {
                    None => {
                        let client = &mut clients.get_mut(event.executor).unwrap().0;
                        client.send_chat_message(format!("Could not find target: {}", raw_target));
                    }
                    Some(target_entity) => {
                        let target_pos = clients.get(target_entity.0).unwrap().1 .0;
                        let pos = &mut clients.get_mut(event.executor).unwrap().1 .0;
                        pos.x = target_pos.x;
                        pos.y = target_pos.y;
                        pos.z = target_pos.z;
                    }
                }

                let client = &mut clients.get_mut(event.executor).unwrap().0;
                client.send_chat_message(format!(
                    "Teleport command executor -> target executed with data:\n {:#?}",
                    &event.result
                ));
            }
            Teleport::TargetToTarget { from, to } => {
                let from_raw_target = match from {
                    EntitySelector::SimpleSelector(x) => match
                    x {
                        EntitySelectors::SinglePlayer(x) => x,
                        _ => "not implemented",
                    },
                    _ => "not implemented",
                };
                let from_target = usernames.iter().find(|(_, name)| name.0 == *from_raw_target);
                let to_raw_target = match to {
                    EntitySelector::SimpleSelector(x) => match
                    x {
                        EntitySelectors::SinglePlayer(x) => &x,
                        _ => "not implemented",
                    },
                    _ => "not implemented",
                };
                let to_target = usernames.iter().find(|(_, name)| name.0 == *to_raw_target);

                let client = &mut clients.get_mut(event.executor).unwrap().0;
                client.send_chat_message(format!(
                    "Teleport command target -> location with data:\n {:#?}",
                    &event.result
                ));
                match from_target {
                    None => {
                        client.send_chat_message(format!("Could not find target: {}", from_raw_target));
                    }
                    Some(from_target_entity) => match to_target {
                        None => {
                            client.send_chat_message(format!("Could not find target: {}", to_raw_target));
                        }
                        Some(to_target_entity) => {
                            let target_pos = *clients.get(to_target_entity.0).unwrap().1;
                            let (from_client, from_pos) =
                                &mut clients.get_mut(from_target_entity.0).unwrap();
                            from_pos.0 = target_pos.0;

                            from_client.send_chat_message(format!(
                                "You have been teleported to {}",
                                to_target_entity.1
                            ));

                            let to_client = &mut clients.get_mut(to_target_entity.0).unwrap().0;
                            to_client.send_chat_message(format!(
                                "{} has been teleported to your location",
                                from_target_entity.1
                            ));
                        }
                    },
                }
            }
            Teleport::TargetToLocation {
                target,
                location,
            } => {
                let raw_target = match target {
                    EntitySelector::SimpleSelector(x) => match
                    x {
                        EntitySelectors::SinglePlayer(x) => x,
                        _ => "not implemented",
                    },
                    _ => "not implemented",
                };
                let target = usernames.iter().find(|(_, name)| name.0 == *raw_target);

                let client = &mut clients.get_mut(event.executor).unwrap().0;
                client.send_chat_message(format!(
                    "Teleport command target -> location with data:\n {:#?}",
                    &event.result
                ));
                match target {
                    None => {
                        client.send_chat_message(format!("Could not find target: {}", raw_target));
                    }
                    Some(target_entity) => {
                        let (client, pos) = &mut clients.get_mut(target_entity.0).unwrap();
                        pos.0.x = location.x.get(pos.0.x as f32) as f64;
                        pos.0.y = location.y.get(pos.0.y as f32) as f64;
                        pos.0.z = location.z.get(pos.0.z as f32) as f64;

                        client.send_chat_message(format!(
                            "Teleport command executor -> location executed with data:\n {:#?}",
                            &event.result
                        ));
                    }
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    mut permissions: ResMut<CommandScopeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            layer
                .chunk
                .set_block([x, SPAWN_Y, z], BlockState::GRASS_BLOCK);
        }
    }

    permissions.add_scope("valence:command:teleport");

    commands.spawn(layer);
}

fn init_clients(
    mut clients: Query<
        (
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut CommandScopes,
            &mut Position,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut permissions,
        mut pos,
        mut game_mode,
    ) in &mut clients
    {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);

        pos.0 = [0.0, SPAWN_Y as f64 + 1.0, 0.0].into();
        *game_mode = GameMode::Creative;

        permissions.add("valence:command:teleport");
    }
}

fn toggle_perms_on_sneak(
    mut clients: Query<&mut CommandScopes>,
    mut events: EventReader<SneakEvent>,
) {
    for event in events.iter() {
        let Ok(mut perms) = clients.get_mut(event.client) else {
            continue;
        };
        if event.state == SneakState::Start {
            match perms.scopes.len() {
                0 => perms.add("valence:command:teleport"),
                1 => perms.remove("valence:command:teleport"),
                _ => panic!("Too many permissions!"),
            };
        }
    }
}
