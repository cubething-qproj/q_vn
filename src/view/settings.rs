//! View settings.

use std::{str::FromStr, time::Duration};

use bevy::platform::collections::HashMap;
use strum::{EnumIter, EnumString, IntoEnumIterator};

use crate::prelude::*;

/// Tunable defaults for view-layer behaviour (stage transitions, etc.).
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct ViewSettings {
    /// Duration of a `pop` transition, in seconds.
    pub pop_duration_secs: f32,
    /// Vertical offset (percent of stage height) used as the start of a `pop`.
    pub pop_offset_pct: f32,
    /// Default duration of a `fade` transition when unspecified, in seconds.
    pub fade_default_secs: f32,
    pub default_font: TextFont,
    // TODO: Query this when the camera changes. Should be easy enough to keep in sync.
    pub logical_viewport_size: Vec2,
    pub comma_delay: Duration,
    pub period_delay: Duration,
}

#[derive(Debug, EnumString, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub enum DelayKind {
    #[strum(to_string = "!")]
    ExclamationPoint,
    #[strum(to_string = "?")]
    QuestionMark,
    #[strum(to_string = ".")]
    Period,
    #[strum(to_string = ",")]
    Comma,
    #[strum(to_string = ";")]
    Semicolon,
    #[strum(to_string = ":")]
    Colon,
    // Any other character
    Character,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub enum CharacterDelay {
    /// 1 second
    Full,
    /// 2/3 = 0.666 seconds
    TwoThirds,
    /// 1/2 = 0.5 a seconds
    Half,
    /// 1/3 = 0.333 seconds
    Third,
    /// 1/4 = 0.25 seconds
    Quarter,
    /// 1/8 = 0.125 seconds
    Eighth,
    /// 1/16 = 0.0625 seconds
    Sixteenth,
    /// 0 seconds
    Zero,
}
impl CharacterDelay {
    pub fn to_f32(self) -> f32 {
        match self {
            CharacterDelay::Full => 1.,
            CharacterDelay::TwoThirds => 0.666,
            CharacterDelay::Half => 0.5,
            CharacterDelay::Third => 0.333,
            CharacterDelay::Quarter => 0.25,
            CharacterDelay::Eighth => 0.125,
            CharacterDelay::Sixteenth => 0.0625,
            CharacterDelay::Zero => 0.,
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
pub struct TypewriterSettings {
    /// Map between the various kinds of punctuation and how long we should wait when we encounter them.
    pub delay_map: HashMap<DelayKind, CharacterDelay>,
    /// How long a full delay is.
    pub full_delay: Duration,
}
impl Default for TypewriterSettings {
    fn default() -> Self {
        Self {
            delay_map: HashMap::from_iter([
                (DelayKind::ExclamationPoint, CharacterDelay::Full),
                (DelayKind::Period, CharacterDelay::Full),
                (DelayKind::QuestionMark, CharacterDelay::Full),
                (DelayKind::Semicolon, CharacterDelay::TwoThirds),
                (DelayKind::Colon, CharacterDelay::TwoThirds),
                (DelayKind::Comma, CharacterDelay::Third),
                (DelayKind::Character, CharacterDelay::Zero),
            ]),
            full_delay: Duration::from_secs_f32(0.6),
        }
    }
}
impl TypewriterSettings {
    pub fn get_timer(&self, kind: DelayKind) -> Timer {
        Timer::from_seconds(
            self.full_delay.as_secs_f32() * self.delay_map.get(&kind).unwrap().to_f32(),
            TimerMode::Once,
        )
    }
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            pop_duration_secs: 0.25,
            pop_offset_pct: 10.,
            fade_default_secs: 0.5,
            default_font: TextFont::default().with_font_size(24.),
            logical_viewport_size: Vec2::new(1920., 1080.),
            comma_delay: Duration::from_secs_f32(0.25),
            period_delay: Duration::from_secs_f32(0.6),
        }
    }
}

pub mod prelude {
    pub use super::ViewSettings;
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ViewSettings>()
        .register_type::<ViewSettings>();
    app.init_resource::<TypewriterSettings>()
        .register_type::<TypewriterSettings>();
}
