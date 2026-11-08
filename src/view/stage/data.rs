use crate::prelude::*;
use strum::EnumString;

/// Accepted easing types.
#[derive(Copy, Clone, Debug, EnumString, Default, Reflect, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Easing {
    BackIn,
    BackInOut,
    BackOut,
    BounceIn,
    BounceInOut,
    BounceOut,
    CircularIn,
    CircularInOut,
    CircularOut,
    CubicIn,
    CubicInOut,
    CubicOut,
    ElasticIn,
    ElasticInOut,
    ElasticOut,
    ExponentialIn,
    ExponentialInOut,
    ExponentialOut,
    #[default]
    Linear,
    QuadraticIn,
    QuadraticInOut,
    QuadraticOut,
    QuarticIn,
    QuarticInOut,
    QuarticOut,
    QuinticIn,
    QuinticInOut,
    QuinticOut,
    SineIn,
    SineInOut,
    SineOut,
    SmoothStep,
    SmoothStepIn,
    SmoothStepOut,
    SmootherStep,
    SmootherStepIn,
    SmootherStepOut,
}
impl Easing {
    pub fn sample(&self, t: f32) -> Option<f32> {
        match self {
            Easing::BackIn => BackInCurve.sample(t),
            Easing::BackInOut => BackInOutCurve.sample(t),
            Easing::BackOut => BackOutCurve.sample(t),
            Easing::BounceIn => BounceInCurve.sample(t),
            Easing::BounceInOut => BounceInOutCurve.sample(t),
            Easing::BounceOut => BounceOutCurve.sample(t),
            Easing::CircularIn => CircularInCurve.sample(t),
            Easing::CircularInOut => CircularInOutCurve.sample(t),
            Easing::CircularOut => CircularOutCurve.sample(t),
            Easing::CubicIn => CubicInCurve.sample(t),
            Easing::CubicInOut => CubicInOutCurve.sample(t),
            Easing::CubicOut => CubicOutCurve.sample(t),
            Easing::ElasticIn => ElasticInCurve.sample(t),
            Easing::ElasticInOut => ElasticInOutCurve.sample(t),
            Easing::ElasticOut => ElasticOutCurve.sample(t),
            Easing::ExponentialIn => ExponentialInCurve.sample(t),
            Easing::ExponentialInOut => ExponentialInOutCurve.sample(t),
            Easing::ExponentialOut => ExponentialOutCurve.sample(t),
            Easing::Linear => LinearCurve.sample(t),
            Easing::QuadraticIn => QuadraticInCurve.sample(t),
            Easing::QuadraticInOut => QuadraticInOutCurve.sample(t),
            Easing::QuadraticOut => QuadraticOutCurve.sample(t),
            Easing::QuarticIn => QuarticInCurve.sample(t),
            Easing::QuarticInOut => QuarticInOutCurve.sample(t),
            Easing::QuarticOut => QuarticOutCurve.sample(t),
            Easing::QuinticIn => QuinticInCurve.sample(t),
            Easing::QuinticInOut => QuinticInOutCurve.sample(t),
            Easing::QuinticOut => QuinticOutCurve.sample(t),
            Easing::SineIn => SineInCurve.sample(t),
            Easing::SineInOut => SineInOutCurve.sample(t),
            Easing::SineOut => SineOutCurve.sample(t),
            Easing::SmoothStep => SmoothStepCurve.sample(t),
            Easing::SmoothStepIn => SmoothStepInCurve.sample(t),
            Easing::SmoothStepOut => SmoothStepOutCurve.sample(t),
            Easing::SmootherStep => SmootherStepCurve.sample(t),
            Easing::SmootherStepIn => SmootherStepInCurve.sample(t),
            Easing::SmootherStepOut => SmootherStepOutCurve.sample(t),
        }
    }
}

/// Accepted stage positions.
/// Used to deserialize from yarn scripts.
#[derive(Reflect, Debug, Default, Clone, PartialEq, Eq, EnumString, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum StagePosition {
    OffLeft,
    FarLeft,
    #[default]
    Left,
    Center,
    Right,
    FarRight,
    OffRight,
}
impl StagePosition {
    /// Matches the position to [UICoord] (left, bottom)
    pub fn to_coord(self) -> UiCoord {
        match self {
            StagePosition::OffLeft => UiCoord::new(0., -25.),
            StagePosition::FarLeft => UiCoord::new(0., 0.),
            StagePosition::Left => UiCoord::new(0., 25.),
            StagePosition::Center => UiCoord::new(0., 50.),
            StagePosition::Right => UiCoord::new(0., 75.),
            StagePosition::FarRight => UiCoord::new(0., 100.),
            StagePosition::OffRight => UiCoord::new(0., 125.),
        }
    }
}

/// Names of the accepted stage commands.
/// Used to deserialize from yarn scripts.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum StageCommand {
    Enter,
    Exit,
    SetSprite,
    SetFocused,
    Move,
    Bg,
    Fx,
}
