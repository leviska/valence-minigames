use crate::{
    area::Area,
    classes::{ArcherClass, ClassName, GameClass, MageClass, RogueClass, WarriorClass},
};
use std::{collections::HashMap, marker::PhantomData, path::PathBuf, str::FromStr};
use valence::{
    anvil::parsing::{DimensionFolder, ParseChunkError},
    entity::text_display::TextDisplayEntityBundle,
    nbt::value::ValueRef,
    prelude::*,
    text::TextContent,
    DEFAULT_TPS,
};

pub fn load_level(
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

#[derive(Component)]
pub struct LobbyLayer;

#[derive(Component)]
pub struct LobbyPlayer;

#[derive(Component)]
pub struct ArenaLayer;

#[derive(Component)]
pub struct ArenaPlayer;

pub fn extract_text_from_sign_nbt(nbt: &Compound) -> Option<Text> {
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

pub fn create_class_text(
    layer_id: Entity,
    layer: &mut LayerBundle,
    area: &Area,
    commands: &mut Commands,
) {
    const SIGNS: [BlockState; 4] = [
        BlockState::OAK_SIGN,
        BlockState::OAK_WALL_SIGN,
        BlockState::OAK_HANGING_SIGN,
        BlockState::OAK_WALL_HANGING_SIGN,
    ];
    for pos in area.iter_block_pos() {
        let Some(block) = layer.chunk.block(pos) else {
            continue;
        };
        if !SIGNS.contains(&block.state) {
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

#[derive(Component)]
pub struct ClassTrigger<Class> {
    pub area: Area,
    _class: PhantomData<Class>,
}

pub fn create_class_trigger(layer: &mut LayerBundle, area: &Area, commands: &mut Commands) {
    let mut triggers: HashMap<BlockState, Option<Area>> = HashMap::from([
        (BlockState::LIGHT_GRAY_WOOL, None),
        (BlockState::ORANGE_WOOL, None),
        (BlockState::RED_WOOL, None),
        (BlockState::LIGHT_BLUE_WOOL, None),
    ]);
    for pos in area.iter_block_pos() {
        let Some(block) = layer.chunk.block(pos) else {
            continue;
        };
        let Some(trigger) = triggers.get_mut(&block.state) else {
            continue;
        };
        let block_area = Area::new(pos, pos);
        let new_trigger = match trigger {
            Some(t) => t.merge(&block_area),
            None => block_area,
        };
        *trigger = Some(new_trigger);
    }
    let mut triggers: HashMap<_, _> = triggers.into_iter().map(|(k, v)| (k, v.unwrap())).collect();
    for (_, area) in triggers.iter_mut() {
        *area = area.expand(BlockPos::new(0, 3, 0));
    }
    commands.spawn(ClassTrigger::<WarriorClass> {
        area: triggers.remove(&BlockState::LIGHT_GRAY_WOOL).unwrap(),
        _class: PhantomData,
    });
    commands.spawn(ClassTrigger::<ArcherClass> {
        area: triggers.remove(&BlockState::ORANGE_WOOL).unwrap(),
        _class: PhantomData,
    });
    commands.spawn(ClassTrigger::<MageClass> {
        area: triggers.remove(&BlockState::RED_WOOL).unwrap(),
        _class: PhantomData,
    });
    commands.spawn(ClassTrigger::<RogueClass> {
        area: triggers.remove(&BlockState::LIGHT_BLUE_WOOL).unwrap(),
        _class: PhantomData,
    });
}

pub fn do_class_triggers<Class: Component + GameClass>(
    mut clients: Query<(Entity, &Position), (With<Client>, With<LobbyPlayer>)>,
    trigger: Query<&ClassTrigger<Class>>,
    mut commands: Commands,
) {
    let trigger = trigger.single();
    for (e, pos) in clients.iter_mut() {
        if trigger.area.contains(pos.0) {
            commands.entity(e).remove::<LobbyPlayer>().insert((
                ArenaPlayer,
                Class::default(),
                ClassName(Class::name()),
            ));
        }
    }
}

pub fn move_to_arena(
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut Position,
            &ClassName,
        ),
        Added<ArenaPlayer>,
    >,
    arena: Query<Entity, (With<ChunkLayer>, With<EntityLayer>, With<ArenaLayer>)>,
    mut commands: Commands,
) {
    let arena = arena.single();
    for (
        e,
        mut client,
        mut entity_layer,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        class_name,
    ) in clients.iter_mut()
    {
        entity_layer.0 = arena;
        visible_chunk_layer.0 = arena;
        visible_entity_layers.0.clear();
        visible_entity_layers.0.insert(arena);

        pos.set([0.0, 61.0, 0.0]);
        commands
            .entity(e)
            .insert((ChunksLoading::default(), KeepPosition(pos.0)));
        client.send_chat_message("You've picked ".into_text() + class_name.0.bold() + " class!");
    }
}

#[derive(Component)]
pub struct ChunksLoading {
    pub timer: i64,
}

impl Default for ChunksLoading {
    fn default() -> Self {
        Self {
            timer: DEFAULT_TPS.get() as i64 / 5,
        }
    }
}

pub fn update_changed_chunk_layer_timer(
    mut timers: Query<(Entity, &mut ChunksLoading)>,
    mut commands: Commands,
) {
    for (e, mut timer) in timers.iter_mut() {
        timer.timer -= 1;
        if timer.timer <= 0 {
            commands.entity(e).remove::<(ChunksLoading, KeepPosition)>();
        }
    }
}

#[derive(Component)]
pub struct KeepPosition(pub DVec3);

pub fn keep_position_while_chunks_loading(
    mut clients: Query<(&KeepPosition, &mut Position), With<ChunksLoading>>,
) {
    for (target, mut pos) in &mut clients {
        pos.set(target.0);
    }
}

pub fn update_inventory_while_chunks_loading(
    mut clients: Query<&mut Inventory, With<ChunksLoading>>,
) {
    for mut inv in &mut clients {
        inv.changed = u64::MAX;
    }
}
