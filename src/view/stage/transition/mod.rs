pub mod data;
pub mod systems;

pub mod prelude {
    pub use super::data::*;
}

use crate::prelude::*;
pub fn plugin(app: &mut App) {
    app.add_plugins(systems::plugin);
}
