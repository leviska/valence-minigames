use valence::prelude::*;

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
