pub mod assets;
pub mod character;
pub mod settings;
pub mod stage;
pub mod ui;
pub mod yarn_events;

pub mod prelude {
    pub use super::YarnSpinnerViewPlugin;
    pub use super::assets::prelude::*;
    pub use super::character::*;
    pub use super::settings::prelude::*;
    pub use super::stage::prelude::*;
    pub use super::ui::prelude::*;
    pub use bevy::prelude::*;
    pub use bevy_yarnspinner::prelude::*;
    pub use tiny_bail::prelude::*;
}
use prelude::*;

pub struct YarnSpinnerViewPlugin;

impl Plugin for YarnSpinnerViewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<YarnSpinnerPlugin>() {
            app.add_plugins(YarnSpinnerPlugin::default());
        }

        app.add_plugins((
            settings::plugin,
            yarn_events::plugin,
            stage::plugin,
            ui::plugin,
            assets::plugin,
        ));
    }
}
