pub use crate::prelude::*;

/// syntax:
/// enter <who> <sprite> <from> <transition>
/// Handler: [super::systems::on_enter]
#[derive(Reflect, Debug, Message, Clone, PartialEq)]
pub struct EnterMsg {
    /// Which character?
    pub who: CharacterId,
    /// With which sprite?
    pub sprite: String,
    /// From where?
    pub from: StagePosition,
    /// With which transition?
    pub transition: StageTransitionKind,
    pub transition_name: StageTransitionName,
}

/// syntax:
/// exit <who> <where> <transition>
/// Handler: [super::systems::on_exit]
#[derive(Reflect, Debug, EntityEvent, Clone, PartialEq)]
pub struct ExitEvent {
    pub entity: Entity,
    pub transition: StageTransitionKind,
}

/// syntax:
/// set_sprite <who> <sprite> <transition>
/// Handler: [super::systems::on_set_sprite]
#[derive(Reflect, Debug, EntityEvent, Clone, PartialEq)]
pub struct SetSpriteEvent {
    /// Which character?
    pub entity: Entity,
    /// To switch sprite?
    pub sprite: String,
    /// With which transition?
    pub transition: StageTransitionKind,
}

/// Focusing refers to whether a character is placed in the front of the stage,
/// in full brightness, or moved slightly backwards and slightly darkened.
/// syntax:
/// set_focused <who> <focused:bool>
/// Handler: [super::systems::on_set_focused]
#[derive(Reflect, Debug, EntityEvent, Clone, PartialEq, Eq)]
pub struct SetFocusedEvent {
    /// Which character?
    pub entity: Entity,
    /// Are they focused?
    pub focused: bool,
}

/// syntax:
/// bg <file>
/// Handler: [super::systems::on_bg]
#[derive(Reflect, Debug, Message, Clone, PartialEq, Eq)]
pub struct BgMsg {
    /// Which file?
    pub file: String,
}

/// syntax:
/// fx <name>
/// Handler: [super::systems::on_fx]
#[derive(Reflect, Debug, Message, Clone, PartialEq, Eq)]
pub struct FxMsg {
    pub name: String,
}
