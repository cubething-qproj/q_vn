//! The stage is where the action takes place.
//! Characters are spawned here, FX happens here, and the background is set.
//! This module covers custom yarn commands.

use crate::prelude::*;

pub mod custom_commands;
pub mod data;
pub mod systems;
pub mod transition;

pub mod prelude {
    pub use super::custom_commands::*;
    pub use super::data::*;
    pub use super::systems::{on_bg, on_enter, on_exit, on_fx, on_set_focused, on_set_sprite};
    pub use super::transition::prelude::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((transition::plugin, systems::plugin));
}
