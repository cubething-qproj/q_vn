use crate::prelude::*;

pub mod character;
pub mod interaction;
pub mod scene;
pub mod textbox;
pub mod typewriter;
pub mod util;

pub mod prelude {
    pub use super::character::*;
    pub use super::interaction::*;
    pub use super::scene::*;
    pub use super::textbox::*;
    pub use super::typewriter::prelude::*;
    pub use super::util::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((interaction::plugin, typewriter::plugin));
}
