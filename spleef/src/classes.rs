use crate::{
    area::Area,
    level::{ArenaLayer, WOOL},
};
use bevy_ecs::query::WorldQuery;
use std::collections::HashSet;
use valence::{
    entity::{
        arrow::ArrowEntityBundle,
        attributes::EntityAttributes,
        egg::EggEntityBundle,
        entity::{Flags, NoGravity},
        thrown_item::Item,
        EntityAttribute, Velocity,
    },
    interact_block::InteractBlockEvent,
    interact_item::InteractItemEvent,
    inventory::{player_inventory::PlayerInventory, HeldItem},
    math::{IVec3, Vec3Swizzles},
    prelude::*,
    DEFAULT_TPS,
};

#[derive(Component)]
pub struct ClassName(pub &'static str);

pub trait GameClass: Default {
    fn name() -> &'static str;
}

#[derive(Component, Default)]
pub struct WarriorClass;

impl GameClass for WarriorClass {
    fn name() -> &'static str {
        "Warrior"
    }
}

#[derive(Component, Default)]
pub struct ArcherClass;

impl GameClass for ArcherClass {
    fn name() -> &'static str {
        "Archer"
    }
}

#[derive(Component, Default)]
pub struct MageClass;

impl GameClass for MageClass {
    fn name() -> &'static str {
        "Mage"
    }
}

#[derive(Component, Default)]
pub struct RogueClass;

impl GameClass for RogueClass {
    fn name() -> &'static str {
        "Rogue"
    }
}

fn clear_inventory(inv: &mut Inventory) {
    for slot in 0..inv.slot_count() {
        inv.set_slot(slot, ItemStack::EMPTY);
    }
}

pub fn init_warrior(
    mut clients: Query<(&mut Inventory, &mut GameMode), (With<Client>, Added<WarriorClass>)>,
) {
    for (mut inv, mut game_mode) in clients.iter_mut() {
        *game_mode = GameMode::Survival;

        let inv = inv.as_mut();
        clear_inventory(inv);

        inv.set_slot(
            PlayerInventory::hotbar_to_slot(0),
            ItemStack::new(ItemKind::WoodenShovel, 1, None),
        );
    }
}

pub fn init_archer(mut clients: Query<&mut Inventory, (With<Client>, Added<ArcherClass>)>) {
    for mut inv in clients.iter_mut() {
        let inv = inv.as_mut();
        clear_inventory(inv);
        inv.set_slot(
            PlayerInventory::hotbar_to_slot(0),
            ItemStack::new(ItemKind::Bow, 1, None),
        );
    }
}

pub fn init_mage(mut clients: Query<&mut Inventory, (With<Client>, Added<MageClass>)>) {
    for mut inv in clients.iter_mut() {
        let inv = inv.as_mut();
        clear_inventory(inv);
        inv.set_slot(
            PlayerInventory::hotbar_to_slot(0),
            ItemStack::new(ItemKind::FireworkRocket, 1, None),
        );
    }
}

pub fn init_rogue(
    mut clients: Query<(&mut Inventory, &mut EntityAttributes), (With<Client>, Added<RogueClass>)>,
) {
    for (mut inv, mut attr) in clients.iter_mut() {
        attr.set_base_value(EntityAttribute::GenericMovementSpeed, 0.2);
        let inv = inv.as_mut();
        clear_inventory(inv);
        inv.set_slot(
            PlayerInventory::hotbar_to_slot(0),
            ItemStack::new(ItemKind::WoodenSword, 1, None),
        );
    }
}

#[derive(Component)]
pub struct Cooldown(pub i32);

pub fn update_cooldown(mut clients: Query<(Entity, &mut Cooldown)>, mut commands: Commands) {
    for (e, mut c) in clients.iter_mut() {
        c.0 -= 1;
        if c.0 <= 0 {
            commands.entity(e).remove::<Cooldown>();
        }
    }
}

pub fn warrior_dig(
    mut clients: Query<
        (&HeldItem, &Inventory),
        (With<Client>, With<WarriorClass>, Without<Cooldown>),
    >,
    mut digging: EventReader<DiggingEvent>,
    mut arena: Query<&mut ChunkLayer, With<ArenaLayer>>,
    mut commands: Commands,
) {
    let mut arena = arena.single_mut();
    let mut processed: HashSet<Entity> = Default::default();
    for event in digging.read() {
        let Ok((held, inv)) = clients.get_mut(event.client) else {
            continue;
        };
        let slot = inv.slot(held.slot());
        if slot.item != ItemKind::WoodenShovel {
            continue;
        }
        let Some(block) = arena.block(event.position) else {
            continue;
        };
        if WOOL.contains(&block.state) {
            if !processed.insert(event.client) {
                continue;
            }
            arena.set_block(event.position, BlockState::AIR);
            commands.entity(event.client).insert(Cooldown(1));
        }
    }
}

#[derive(Component)]
pub struct ArcherArrow;

pub fn arrow_movement(mut arrows: Query<(&mut Position, &mut Velocity), With<ArcherArrow>>) {
    for (mut pos, mut vel) in arrows.iter_mut() {
        // should be somewhat similar to minecraft physics
        // so client doesn't jitter too much
        vel.0.y -= 1.0;
        vel.0 *= 0.99;
        let vel: DVec3 = vel.0.into();
        pos.0 += vel / 20.0;
    }
}

pub fn arrow_oob(arrows: Query<(Entity, &Position), With<ArcherArrow>>, mut commands: Commands) {
    let bounds = Area::new(BlockPos::new(-100, 0, -100), BlockPos::new(100, 500, 100));
    for (e, pos) in arrows.iter() {
        let block_pos: BlockPos = pos.0.into();
        if !bounds.contains(block_pos) {
            commands.entity(e).insert(Despawned);
        }
    }
}

pub fn arrow_intersection(
    arrows: Query<(Entity, &Position, &Velocity), With<ArcherArrow>>,
    mut arena: Query<&mut ChunkLayer, With<ArenaLayer>>,
    mut commands: Commands,
) {
    let mut arena = arena.single_mut();
    for (e, pos, vel) in arrows.iter() {
        let mut last_block_pos: Option<BlockPos> = None;
        // too lazy to raycast and this works fine-ish
        for i in 0..10 {
            let block_pos: BlockPos = (pos.0 + (vel.0 * i as f32 / 200.0).as_dvec3()).into();
            if let Some(last_block_pos) = last_block_pos {
                if block_pos == last_block_pos {
                    continue;
                }
            }
            last_block_pos = Some(block_pos);
            let Some(block) = arena.block(block_pos) else {
                continue;
            };
            if block.state == BlockState::AIR {
                continue;
            }
            if WOOL.contains(&block.state) {
                arena.set_block(block_pos, BlockState::AIR);
            }
            commands.entity(e).insert(Despawned);
            break;
        }
    }
}

pub fn archer_shoot(
    mut clients: Query<
        (&HeldItem, &Inventory, &Position, &Look, &EntityLayerId),
        (With<Client>, With<ArcherClass>, Without<Cooldown>),
    >,
    mut interacts: EventReader<InteractItemEvent>,
    mut commands: Commands,
) {
    let mut processed: HashSet<Entity> = Default::default();
    for event in interacts.read() {
        let Ok((held, inv, pos, look, entity_layer)) = clients.get_mut(event.client) else {
            continue;
        };
        let slot = inv.slot(held.slot());
        if slot.item != ItemKind::Bow {
            continue;
        }
        if !processed.insert(event.client) {
            continue;
        }
        let shift: DVec3 = [0.0, 1.5, 0.0].into();
        let arrow_origin = pos.0 + shift + look.vec().as_dvec3();
        commands.spawn((
            ArrowEntityBundle {
                entity_no_gravity: NoGravity(false),
                position: Position(arrow_origin),
                velocity: Velocity(look.vec() * 30.0),
                layer: *entity_layer,
                ..Default::default()
            },
            ArcherArrow,
        ));
        commands.entity(event.client).insert(Cooldown(5));
    }
}

#[derive(Component)]
pub struct MageFireball;

pub fn fireball_movement(mut arrows: Query<(&mut Position, &Velocity), With<MageFireball>>) {
    for (mut pos, vel) in arrows.iter_mut() {
        let vel: DVec3 = vel.0.into();
        pos.0 += vel / 20.0;
    }
}

pub fn fireball_oob(
    arrows: Query<(Entity, &Position), With<MageFireball>>,
    mut commands: Commands,
) {
    let bounds = Area::new(BlockPos::new(-100, 0, -100), BlockPos::new(100, 100, 100));
    for (e, pos) in arrows.iter() {
        let block_pos: BlockPos = pos.0.into();
        if !bounds.contains(block_pos) {
            commands.entity(e).insert(Despawned);
        }
    }
}

pub fn fireball_intersection(
    arrows: Query<(Entity, &Position, &Velocity), With<MageFireball>>,
    mut arena: Query<&mut ChunkLayer, With<ArenaLayer>>,
    mut commands: Commands,
) {
    let mut arena = arena.single_mut();
    for (e, pos, vel) in arrows.iter() {
        let mut last_block_pos: Option<BlockPos> = None;
        // too lazy to raycast and this works fine-ish
        for i in 0..10 {
            let block_pos: BlockPos = (pos.0 + (vel.0 * i as f32 / 200.0).as_dvec3()).into();
            if let Some(last_block_pos) = last_block_pos {
                if block_pos == last_block_pos {
                    continue;
                }
            }
            last_block_pos = Some(block_pos);
            let Some(block) = arena.block(block_pos) else {
                continue;
            };
            if block.state == BlockState::AIR {
                continue;
            }
            // collision
            for shift in [
                (-1, 0, 0),
                (0, -1, 0),
                (0, 0, -1),
                (1, 0, 0),
                (0, 1, 0),
                (0, 0, 1),
                (0, 0, 0),
            ] {
                let shift: IVec3 = shift.into();
                let blasted_block_pos = block_pos + shift;
                let Some(blasted_block) = arena.block(blasted_block_pos) else {
                    continue;
                };
                if WOOL.contains(&blasted_block.state) {
                    arena.set_block(blasted_block_pos, BlockState::AIR);
                }
            }
            commands.entity(e).insert(Despawned);
            break;
        }
    }
}

pub fn mage_shoot(
    mut clients: Query<
        (&HeldItem, &Inventory, &Position, &Look, &EntityLayerId),
        (With<Client>, With<MageClass>, Without<Cooldown>),
    >,
    mut item_interacts: EventReader<InteractItemEvent>,
    mut block_interacts: EventReader<InteractBlockEvent>,
    mut commands: Commands,
) {
    let mut processed: HashSet<Entity> = Default::default();
    let mut process = |client: Entity| {
        let Ok((held, inv, pos, look, entity_layer)) = clients.get_mut(client) else {
            return;
        };
        let slot = inv.slot(held.slot());
        if slot.item != ItemKind::FireworkRocket {
            return;
        }
        if !processed.insert(client) {
            return;
        }
        let shift: DVec3 = [0.0, 1.5, 0.0].into();
        let arrow_origin = pos.0 + shift + look.vec().as_dvec3();
        // Egg is nice, because it doesn't jitter too much on the client and has old version support
        commands.spawn((
            EggEntityBundle {
                thrown_item_item: Item(ItemStack::new(ItemKind::FireCharge, 1, None)),
                entity_no_gravity: NoGravity(true),
                position: Position(arrow_origin),
                velocity: Velocity(look.vec() * 30.0),
                look: *look,
                layer: *entity_layer,
                ..Default::default()
            },
            MageFireball,
        ));
        commands.entity(client).insert(Cooldown(15));
    };
    for event in item_interacts.read() {
        process(event.client);
    }
    for event in block_interacts.read() {
        process(event.client);
    }
}

#[derive(Default, Component)]
pub struct CombatState {
    pub last_attacked_tick: i64,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct CombatQuery {
    client: &'static mut Client,
    state: &'static mut CombatState,
    flags: &'static Flags,
    pos: &'static Position,
    vel: &'static mut Velocity,
}

pub fn combat(
    server: Res<Server>,
    mut clients: Query<CombatQuery>,
    rogues: Query<(&HeldItem, &Inventory), With<RogueClass>>,
    mut interact_entity: EventReader<InteractEntityEvent>,
) {
    for event in interact_entity.read() {
        let Ok([attacker, mut victim]) = clients.get_many_mut([event.client, event.entity]) else {
            continue;
        };
        let current_tick = server.current_tick();
        if current_tick - victim.state.last_attacked_tick < DEFAULT_TPS.get() as i64 / 2 {
            continue;
        }
        victim.state.last_attacked_tick = current_tick;

        let victim_pos = victim.pos.0.xz();
        let attacker_pos = attacker.pos.0.xz();

        let dir = (victim_pos - attacker_pos).normalize().as_vec2();

        let bonus_knockback = if let Ok((slot, inv)) = rogues.get(event.client) {
            if inv.slot(slot.slot()).item == ItemKind::WoodenSword {
                1.2
            } else {
                1.0
            }
        } else {
            1.0
        };
        let knockback_xz = if attacker.flags.sprinting() {
            18.0
        } else {
            8.0
        } * bonus_knockback;
        let knockback_y = if attacker.flags.sprinting() {
            8.432
        } else {
            6.432
        };

        let new_vel: Vec3 = [dir.x * knockback_xz, knockback_y, dir.y * knockback_xz].into();
        victim.client.set_velocity(new_vel);
        victim.vel.0 = new_vel;

        // victim.client.trigger_status(EntityStatus::PlayAttackSound);
        // victim.statuses.trigger(EntityStatus::PlayAttackSound);
    }
}
