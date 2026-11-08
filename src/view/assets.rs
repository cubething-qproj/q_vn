//! Load assets from standard locations.

use bevy::{asset::LoadedFolder, platform::collections::HashMap};

use crate::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct FontsFolder(Handle<LoadedFolder>);
impl FontsFolder {
    /// Panics if an asset is found that is _not_ a font.
    pub fn get(&self, name: &str, folders: &Res<Assets<LoadedFolder>>) -> Option<Handle<Font>> {
        let folder = folders.get(&self.0)?;
        folder
            .handles
            .iter()
            .find(|handle| {
                debug!("found font {:?}", handle.path());
                handle
                    .path()
                    .and_then(|path| path.path().file_name()?.to_str())
                    .map(|file_name| file_name.eq_ignore_ascii_case(name))
                    .unwrap_or_default()
            })
            .map(|res| res.clone().typed::<Font>())
    }

    /// Loads fonts from `assets/fonts`. Will load the entire folder.
    pub fn load(asset_server: Res<AssetServer>, mut fonts: ResMut<FontsFolder>) {
        let folder = asset_server.load_folder("fonts");
        fonts.0 = folder;
    }
}

#[derive(Resource, Debug, Default)]
pub struct Backgrounds(Handle<LoadedFolder>);
impl Backgrounds {
    /// Panics if an asset is found that is _not_ an image.
    pub fn get(&self, name: &str, folders: &Res<Assets<LoadedFolder>>) -> Option<Handle<Image>> {
        let folder = folders.get(&self.0)?;
        folder
            .handles
            .iter()
            .find(|handle| {
                handle
                    .path()
                    .and_then(|path| path.path().file_name()?.to_str())
                    .map(|file_name| file_name.eq_ignore_ascii_case(name))
                    .unwrap_or_default()
            })
            .map(|res| res.clone().typed::<Image>())
    }

    /// Loads background images from `assets/backgrounds`. Will load the entire
    /// folder.
    pub fn load(asset_server: Res<AssetServer>, mut backgrounds: ResMut<Backgrounds>) {
        let folder = asset_server.load_folder("backgrounds");
        backgrounds.0 = folder;
    }
}

#[derive(Resource, Debug, Default)]
pub struct CharacterSprites(HashMap<CharacterId, Handle<LoadedFolder>>);
impl CharacterSprites {
    pub fn get_character_sprite(
        &self,
        who: &CharacterId,
        file_name: &str,
        folders: &Res<Assets<LoadedFolder>>,
    ) -> Option<Handle<Image>> {
        let folder = folders.get(self.0.get(who)?)?;
        folder
            .handles
            .iter()
            .find(|handle| {
                handle
                    .path()
                    .and_then(|path| path.path().file_name()?.to_str())
                    .map(|name| name.eq_ignore_ascii_case(file_name))
                    == Some(true)
            })
            .map(|handle| handle.clone().typed::<Image>())
    }

    /// Loads sprites associated with registered characters. Assumes assets are at
    /// `assets/characters/<character_name>` Will load the _entire folder._
    pub fn load(
        registry: Res<CharacterRegistry>,
        asset_server: Res<AssetServer>,
        mut csprites: ResMut<CharacterSprites>,
    ) {
        for character in registry.iter() {
            let folder = asset_server.load_folder(format!("characters/{}", character.name));
            csprites.0.insert(character.clone(), folder);
        }
    }
}

/// Run condition: are all bulk-loaded assets (fonts, backgrounds, character
/// sprites) and the [YarnProject] ready?
pub fn assets_ready(
    registry: Res<CharacterRegistry>,
    characters: Res<CharacterSprites>,
    fonts: Res<FontsFolder>,
    bgs: Res<Backgrounds>,
    server: Res<AssetServer>,
    yarn_proj: Option<Res<YarnProject>>,
) -> bool {
    // Guard against the frame where `load_character_sprites` hasn't yet
    // populated `CharacterSprites` from `CharacterRegistry`; an empty map
    // would otherwise make `.all(...)` trivially true.
    let characters_registered = characters.0.len() == registry.iter().count();
    let characters_ready = characters_registered
        && characters
            .0
            .values()
            .all(|val| server.is_loaded_with_dependencies(val));
    let fonts_ready = server.is_loaded_with_dependencies(&fonts.0);
    let bgs_ready = server.is_loaded_with_dependencies(&bgs.0);
    let yarn_ready = yarn_proj.is_some();
    characters_ready && fonts_ready && bgs_ready && yarn_ready
}

pub mod prelude {
    pub use super::{Backgrounds, CharacterSprites, FontsFolder, assets_ready};
}

// Note: This should probably be adjusted later on to be event-based,
// this way we don't pre-load assets into memory that we may not need.
pub fn plugin(app: &mut App) {
    app.init_resource::<Backgrounds>();
    app.init_resource::<FontsFolder>();
    app.init_resource::<CharacterSprites>();
    app.add_systems(Startup, (Backgrounds::load, FontsFolder::load));
    // This is delayed because the character registry must be populated first via [ `App::register_characters` ]
    app.add_systems(
        Update,
        CharacterSprites::load.run_if(resource_added::<CharacterRegistry>),
    );
}
