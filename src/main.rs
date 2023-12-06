use std::path::PathBuf;

use valence::{
    anvil::parsing::{DimensionFolder, ParseChunkError},
    prelude::*,
};

pub fn load_level(
    path: impl Into<PathBuf>,
    biomes: &BiomeRegistry,
    dimensions: &DimensionTypeRegistry,
    server: &Server,
    area: (impl Into<BlockPos>, impl Into<BlockPos>),
) -> Result<LayerBundle, ParseChunkError> {
    let mut folder = DimensionFolder::new(path, &biomes);
    let area: (ChunkPos, ChunkPos) = (area.0.into().into(), area.1.into().into());
    let area: (ChunkPos, ChunkPos) = (
        ChunkPos::new(area.0.x.min(area.1.x), area.0.z.min(area.1.z)),
        ChunkPos::new(area.0.x.max(area.1.x), area.0.z.max(area.1.z)),
    );
    let mut layer = LayerBundle::new(ident!("overworld"), dimensions, biomes, server);
    for x in area.0.x..=area.1.x {
        for z in area.0.z..=area.1.z {
            let pos = ChunkPos::new(x, z);
            if let Some(chunk) = folder.get_chunk(pos)? {
                layer.chunk.insert_chunk(pos, chunk.chunk);
            }
        }
    }
    Ok(layer)
}

pub fn main() {
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, init_clients)
        .run();
}

#[derive(Component)]
pub struct ArenaLayer;

#[derive(Component)]
pub struct LobbyLayer;

pub fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let layer = load_level(
        "maps/arena",
        &biomes,
        &dimensions,
        &server,
        ([-100, 0, -100], [100, 0, 100]),
    )
    .unwrap();
    commands.spawn((layer, ArenaLayer));

    let layer = load_level(
        "maps/lobby",
        &biomes,
        &dimensions,
        &server,
        ([-100, 0, -100], [100, 0, 100]),
    )
    .unwrap();
    commands.spawn((layer, LobbyLayer));
}

pub fn init_clients(
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut GameMode,
            &mut Position,
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
    ) in clients.iter_mut()
    {
        entity_layer.0 = lobby;
        visible_chunk_layer.0 = lobby;
        visible_entity_layers.0.insert(lobby);

        pos.set([0.0, 61.0, 0.0]);
        *game_mode = GameMode::Adventure;

        client.send_chat_message("Welcome!".italic());
    }
}
