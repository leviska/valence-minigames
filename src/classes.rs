use valence::{inventory::player_slots::HOTBAR_START, prelude::*};

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

fn debug_inventory(inv: &mut Inventory) {
    for slot in 0..inv.slot_count() {
        inv.set_slot(
            slot,
            ItemStack::new(ItemKind::Cobblestone, (slot + 1) as i8, None),
        );
    }
}

pub fn init_warrior(mut clients: Query<&mut Inventory, (With<Client>, Added<WarriorClass>)>) {
    for mut inv in clients.iter_mut() {
        let inv = inv.as_mut();
        clear_inventory(inv);
        inv.set_slot(
            HOTBAR_START,
            ItemStack::new(ItemKind::DiamondShovel, 1, None),
        );
    }
}
