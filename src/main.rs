use area::Area;
use std::{path::PathBuf, str::FromStr};
use valence::{
    anvil::parsing::{DimensionFolder, ParseChunkError},
    entity::text_display::TextDisplayEntityBundle,
    interact_block::InteractBlockEvent,
    nbt::value::ValueRef,
    prelude::*,
    spawn::IsFlat,
    text::TextContent,
};

pub mod area;

const READY_BUTTON_POS: [i32; 3] = [0, 61, 4];

fn load_level(
    path: impl Into<PathBuf>,
    biomes: &BiomeRegistry,
    dimensions: &DimensionTypeRegistry,
    server: &Server,
    area: &Area,
) -> Result<LayerBundle, ParseChunkError> {
    let mut folder = DimensionFolder::new(path, &biomes);
    let mut layer = LayerBundle::new(ident!("overworld"), dimensions, biomes, server);
    for pos in area.iter_chunk_pos() {
        if let Some(chunk) = folder.get_chunk(pos)? {
            layer.chunk.insert_chunk(pos, chunk.chunk);
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
        .add_systems(Update, (init_clients, event_handler, teleporting))
        .run();
}

#[derive(Component)]
pub struct ArenaLayer;

#[derive(Component)]
pub struct LobbyLayer;

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let lobby_area = Area::new([-50, 50, -50], [50, 80, 50]);
    let mut lobby = load_level("maps/lobby", &biomes, &dimensions, &server, &lobby_area).unwrap();
    let lobby_id = commands.spawn(LobbyLayer).id();
    process_class_text(lobby_id, &mut lobby, &lobby_area, &mut commands);
    commands.entity(lobby_id).insert(lobby);

    let arena_area = Area::new([-100, 50, -100], [100, 100, 100]);
    let arena = load_level("maps/arena", &biomes, &dimensions, &server, &arena_area).unwrap();
    commands.spawn((arena, ArenaLayer));
}

fn extract_text_from_sign_nbt(nbt: &Compound) -> Option<Text> {
    let text = nbt.get("front_text").map(|x| x.as_value_ref())?;
    let ValueRef::Compound(text) = text else {
        return None;
    };
    let text = text.get("messages").map(|x| x.as_value_ref())?;
    let ValueRef::List(text) = text else {
        return None;
    };
    let text = text.get(0)?;
    let ValueRef::String(text) = text else {
        return None;
    };
    let text = Text::from_str(text).ok()?;
    Some(text)
}

fn process_class_text(
    layer_id: Entity,
    layer: &mut LayerBundle,
    area: &Area,
    commands: &mut Commands,
) {
    let signs = [
        BlockState::OAK_SIGN,
        BlockState::OAK_WALL_SIGN,
        BlockState::OAK_HANGING_SIGN,
        BlockState::OAK_WALL_HANGING_SIGN,
    ];
    for pos in area.iter_block_pos() {
        let Some(block) = layer.chunk.block(pos) else {
            continue;
        };
        if !signs.contains(&block.state) {
            continue;
        }
        let Some(nbt) = block.nbt else {
            continue;
        };
        let Some(text) = extract_text_from_sign_nbt(nbt) else {
            continue;
        };
        let content = &text.content;
        let TextContent::Text { text } = content else {
            continue;
        };
        let display_text: Text = match text.as_ref() {
            "warrior" => "Warrior Class",
            "archer" => "Archer Class",
            "mage" => "Mage Class",
            "rogue" => "Rogue Class",
            _ => {
                continue;
            }
        }
        .into_text();
        layer.chunk.set_block(pos, BlockState::AIR).unwrap();
        commands.spawn(TextDisplayEntityBundle {
            layer: EntityLayerId(layer_id),
            text_display_text: valence::entity::text_display::Text(display_text),
            position: Position([pos.x as f64 + 0.5, pos.y as f64, pos.z as f64 - 0.5].into()),
            look: Look::new(180.0, 0.0),
            ..Default::default()
        });
    }
}

fn init_clients(
    mut clients: Query<
        (
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
) {
    let lobby = lobby.single();
    for (
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
        *game_mode = GameMode::Adventure;

        client.send_chat_message("Welcome!".italic());
    }
}

#[derive(Component)]
pub struct Teleporting(DVec3);

fn event_handler(
    mut clients: Query<(
        &mut Client,
        &mut EntityLayerId,
        &mut VisibleChunkLayer,
        &mut VisibleEntityLayers,
    )>,
    mut block_interacts: EventReader<InteractBlockEvent>,
    lobby: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<LobbyLayer>)>,
    arena: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<ArenaLayer>)>,
    mut commands: Commands,
) {
    let lobby = lobby.single();
    let arena = arena.single();
    for e in block_interacts.read() {
        if e.hand != Hand::Main {
            continue;
        }
        let Ok((mut client, mut entity_layer, mut visible_chunk_layer, mut visible_entity_layers)) =
            clients.get_mut(e.client)
        else {
            continue;
        };
        if entity_layer.0 != lobby {
            continue;
        }
        if e.position != READY_BUTTON_POS.into() {
            continue;
        }
        entity_layer.0 = arena;
        visible_chunk_layer.0 = arena;
        visible_entity_layers.0.clear();
        visible_entity_layers.0.insert(arena);
        commands
            .entity(e.client)
            .insert(Teleporting([0.0, 61.0, 0.0].into()));
        client.send_chat_message("Ready!");
    }
}

fn teleporting(
    mut clients: Query<(Entity, &Teleporting, &mut Position), With<Client>>,
    mut commands: Commands,
) {
    for (e, tp, mut pos) in &mut clients {
        pos.set(tp.0);
        commands.entity(e).remove::<Teleporting>();
    }
}
