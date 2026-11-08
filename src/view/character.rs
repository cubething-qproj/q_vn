use bevy::platform::collections::HashSet;

use super::prelude::*;

/// All of the characters in the story.
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct CharacterRegistry(HashSet<CharacterId>);

/// A character's ID, used to uniquely identify a [Character]. Registered in
/// [CharacterRegistry] via [App::register_characters].
/// Character names are matched in a case-insensitive manner.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Deref, Reflect)]
pub struct CharacterId {
    pub name: String,
}
impl Default for CharacterId {
    fn default() -> Self {
        Self {
            name: "PLACEHOLDER".to_string(),
        }
    }
}
impl CharacterId {
    pub fn is(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}
impl<S: ToString> From<S> for CharacterId {
    fn from(value: S) -> Self {
        Self {
            name: value.to_string(),
        }
    }
}

/// An actor in a scene. Must be registered in the [CharacterRegistry] via [App::register_characters].
#[derive(Component)]
pub struct Character {
    pub id: CharacterId,
}
impl Character {
    pub fn is(&self, name: &str) -> bool {
        self.id.is(name)
    }
}

pub trait RegisterCharacterAppExt {
    fn register_characters<S: Into<CharacterId>>(
        &mut self,
        character: impl IntoIterator<Item = S>,
    ) -> &mut Self;
}
impl RegisterCharacterAppExt for App {
    /// Register [Character]s to the VN engine via unique [CharacterId]s.
    /// Each [CharacterId] must be unique by hash. Any overlapping IDs will be overwritten.
    fn register_characters<S: Into<CharacterId>>(
        &mut self,
        characters: impl IntoIterator<Item = S>,
    ) -> &mut Self {
        self.init_resource::<CharacterRegistry>();
        self.world_mut()
            .resource_mut::<CharacterRegistry>()
            .extend(characters.into_iter().map(|c| c.into()));
        info!("{:#?}", self.world().resource::<CharacterRegistry>());
        self
    }
}
