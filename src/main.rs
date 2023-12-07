use area::Area;
use classes::{ArcherClass, MageClass, RogueClass, WarriorClass};
use level::{ArenaLayer, LobbyLayer, LobbyPlayer};
use valence::{prelude::*, spawn::IsFlat};

pub mod area;
mod classes;
mod level;

pub fn main() {
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (
                level::update_changed_chunk_layer_timer,
                classes::update_cooldown,
            ),
        )
        .add_systems(
            Update,
            (
                init_clients,
                level::do_class_triggers::<WarriorClass>,
                level::do_class_triggers::<ArcherClass>,
                level::do_class_triggers::<MageClass>,
                level::do_class_triggers::<RogueClass>,
                level::move_to_arena,
                level::keep_position_while_chunks_loading,
                level::update_inventory_while_chunks_loading,
                classes::init_warrior,
                classes::init_archer,
                classes::init_mage,
                classes::init_rogue,
                classes::warrior_dig,
                classes::archer_shoot,
                (
                    (classes::arrow_intersection, classes::arrow_oob),
                    classes::arrow_movement,
                )
                    .chain(),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let lobby_area = Area::new([-50, 50, -50], [50, 80, 50]);
    let mut lobby =
        level::load_level("maps/lobby", &biomes, &dimensions, &server, &lobby_area).unwrap();
    let lobby_id = commands.spawn(LobbyLayer).id();
    level::create_class_text(lobby_id, &mut lobby, &lobby_area, &mut commands);
    level::create_class_trigger(&mut lobby, &lobby_area, &mut commands);
    commands.entity(lobby_id).insert(lobby);

    let arena_area = Area::new([-100, 50, -100], [100, 100, 100]);
    let arena =
        level::load_level("maps/arena", &biomes, &dimensions, &server, &arena_area).unwrap();
    commands.spawn((arena, ArenaLayer));
}

fn init_clients(
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut GameMode,
            &mut Position,
            &mut IsFlat,
        ),
        Added<Client>,
    >,
    lobby: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<LobbyLayer>)>,
    mut commands: Commands,
) {
    let lobby = lobby.single();
    for (
        entity,
        mut client,
        mut entity_layer,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut game_mode,
        mut pos,
        mut is_flat,
    ) in clients.iter_mut()
    {
        entity_layer.0 = lobby;
        visible_chunk_layer.0 = lobby;
        visible_entity_layers.0.insert(lobby);
        is_flat.0 = true;

        pos.set([0.0, 61.0, 0.0]);
        *game_mode = GameMode::Survival;

        commands.entity(entity).insert(LobbyPlayer);

        client.send_chat_message("Welcome to ".into_text() + "Spleef: RPG".italic());
    }
}

// fn event_handler(
//     mut clients: Query<(
//         &mut Client,
//         &mut EntityLayerId,
//         &mut VisibleChunkLayer,
//         &mut VisibleEntityLayers,
//     )>,
//     mut block_interacts: EventReader<InteractBlockEvent>,
//     lobby: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<LobbyLayer>)>,
//     arena: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<ArenaLayer>)>,
//     mut commands: Commands,
// ) {
//     let lobby = lobby.single();
//     let arena = arena.single();
//     for e in block_interacts.read() {
//         if e.hand != Hand::Main {
//             continue;
//         }
//         let Ok((mut client, mut entity_layer, mut visible_chunk_layer, mut visible_entity_layers)) =
//             clients.get_mut(e.client)
//         else {
//             continue;
//         };
//         if entity_layer.0 != lobby {
//             continue;
//         }
//         // if e.position != READY_BUTTON_POS.into() {
//         //     continue;
//         // }
//         entity_layer.0 = arena;
//         visible_chunk_layer.0 = arena;
//         visible_entity_layers.0.clear();
//         visible_entity_layers.0.insert(arena);
//         commands
//             .entity(e.client)
//             .insert(Teleporting([0.0, 61.0, 0.0].into()));
//         client.send_chat_message("Ready!");
//     }
// }
