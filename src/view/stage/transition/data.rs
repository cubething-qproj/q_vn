use std::{str::FromStr, time::Duration};

use bevy::ecs::system::SystemId;
use strum::EnumString;

use crate::prelude::*;

type TransitionCallback = SystemId<In<Entity>, ()>;

// TODO: A trait?
/// indicates that a character sprite is currently sliding.
#[derive(Component, Debug, Clone)]
pub struct SlideTransition {
    pub(crate) from: UiCoord,
    pub(crate) to: UiCoord,
    pub(crate) callback: Option<TransitionCallback>,
    pub(crate) timer: Timer,
    pub(crate) easing: Easing,
}
impl SlideTransition {
    pub fn new(
        from: UiCoord,
        to: UiCoord,
        duration: Duration,
        easing: Easing,
        callback: Option<TransitionCallback>,
    ) -> Self {
        Self {
            from,
            to,
            callback,
            timer: Timer::new(duration, TimerMode::Once),
            easing,
        }
    }
}

/// Indicates that a character sprite is currently fading in or out.
#[derive(Component, Debug, Clone)]
pub struct FadeTransition {
    pub(crate) fade_in: bool,
    pub(crate) callback: Option<TransitionCallback>,
    pub(crate) timer: Timer,
    pub(crate) easing: Easing,
}
impl FadeTransition {
    pub fn new(
        fade_in: bool,
        duration: Duration,
        easing: Easing,
        callback: Option<TransitionCallback>,
    ) -> Self {
        Self {
            fade_in,
            callback,
            timer: Timer::new(duration, TimerMode::Once),
            easing,
        }
    }
}

/// This is used to determine the syntax when deserializing from yarn.
/// See individual values for syntax.
#[derive(EnumString, Reflect, Debug, PartialEq, Eq, Copy, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum StageTransitionName {
    /// syntax: fade <duration?> <easing?>
    Fade,
    /// syntax: slide <from> <to> <duration?> <easing?>
    Slide,
    /// syntax: pop <duration?> <easing?>
    Pop,
    None,
}

/// Kind of stage transition.
/// See [StageTransitionName] for command syntax.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum StageTransitionKind {
    /// Fade out or in. Parsed from [StageTransitionName::Fade]
    // TODO: Use this to blend between sprites
    Fade {
        duration: Duration,
        fade_in: bool,
        easing: Easing,
    },
    /// Slide the sprite into position. Parsed from [StageTransitionName::Slide]
    /// or [StageTransitionName::Pop]
    Slide {
        from: UiCoord,
        to: UiCoord,
        duration: Duration,
        easing: Easing,
    },
    /// No transition
    None,
}
impl StageTransitionKind {
    /// Shorthand for a kind of [Self::Slide]. See [StageTransitionName::Pop]
    pub fn pop(current_pos: UiCoord, settings: &ViewSettings) -> Self {
        Self::Slide {
            from: current_pos.with_bottom(-settings.pop_offset_pct),
            to: current_pos,
            duration: Duration::from_secs_f32(settings.pop_duration_secs),
            easing: Easing::default(),
        }
    }
    pub fn insert(
        &self,
        commands: &mut Commands,
        id: Entity,
        callback: Option<TransitionCallback>,
    ) {
        match *self {
            StageTransitionKind::Fade {
                duration,
                fade_in,
                easing,
            } => {
                let transition = FadeTransition::new(fade_in, duration, easing, callback);
                commands.entity(id).insert(transition);
            }
            StageTransitionKind::Slide {
                from,
                to,
                duration,
                easing,
            } => {
                let transition = SlideTransition::new(from, to, duration, easing, callback);
                commands.entity(id).insert(transition);
            }
            StageTransitionKind::None => {}
        }
    }
}

impl StageTransitionKind {
    pub fn from_yarn(
        params: &[YarnValue],
        current_pos: UiCoord,
        stage_command: StageCommand,
        settings: &ViewSettings,
    ) -> Result<Self> {
        let Some(YarnValue::String(name)) = params.first() else {
            return Err(BevyError::error(
                "Missing or invalid stage transition name.",
            ));
        };
        let Ok(name) = StageTransitionName::from_str(name) else {
            return Err(
                format!("Invalid stage transition name '{name}' (params = {params:?})").into(),
            );
        };
        macro_rules! parse_param {
            ($name:ident, $at:literal) => {
                try_parse_param!($name, $at);
                let $name = match $name {
                    Ok(val) => val.to_string(),
                    Err(e) => {
                        return Err(e.into());
                    }
                };
            };
        }
        macro_rules! try_parse_param {
            ($name:ident, $at:literal) => {
                // Note: Custom command parameters are always parsed as strings.
                let $name = if let Some(YarnValue::String(yarnval)) = params.get($at) {
                    Ok(yarnval)
                } else {
                    Err(format!(
                        "Missing or incorrect parameter '{}' in transition '{}' at position {}",
                        stringify!($name),
                        params
                            .iter()
                            .map(|yv| yv.to_string())
                            .collect::<Vec<_>>()
                            .join(" "),
                        stringify!($at)
                    ))
                };
            };
        }
        macro_rules! parse_duration {
            ($name:ident, $at:literal) => {
                try_parse_param!($name, $at);
                let $name = $name
                    .ok()
                    .and_then(|d| d.parse::<f32>().ok())
                    .unwrap_or(settings.fade_default_secs);
                let $name = Duration::from_secs_f32($name);
            };
        }
        macro_rules! parse_easing {
            ($name:ident, $at:literal, $default:expr) => {
                try_parse_param!($name, $at);
                let $name = if let Ok($name) = $name {
                    if let Ok(val) = $name.parse::<Easing>() {
                        val
                    } else {
                        return Err(format!("Invalid easing value {}", stringify!($name)).into());
                    }
                } else {
                    $default
                };
            };
            ($name:ident, $at:literal) => {
                parse_easing!($name, $at, Easing::default())
            };
        }
        match name {
            StageTransitionName::Fade => {
                parse_duration!(duration, 1);
                parse_easing!(easing, 2);
                let res = match stage_command {
                    StageCommand::Enter => Self::Fade {
                        duration,
                        fade_in: true,
                        easing,
                    },
                    StageCommand::Exit => Self::Fade {
                        duration,
                        fade_in: false,
                        easing,
                    },
                    _ => {
                        // TODO: Maybe loosen this restriction, e.g. to fade to/from focus.
                        return Err(format!("Invalid transition for event {stage_command:?}. Fade may only be applied to Enter, Exit, and SetSprite.").into());
                    }
                };
                Ok(res)
            }
            StageTransitionName::Slide => {
                parse_param!(to, 1);
                let Ok(to) = to.parse::<StagePosition>().map(|to| to.to_coord()) else {
                    return Err(format!("Invalid stage position {}", to).into());
                };

                parse_duration!(duration, 2);
                parse_easing!(easing, 3);

                Ok(Self::Slide {
                    from: current_pos,
                    to,
                    duration,
                    easing,
                })
            }
            StageTransitionName::Pop => {
                parse_duration!(duration, 1);
                parse_easing!(easing, 2, Easing::CubicOut);

                Ok(Self::Slide {
                    from: current_pos.with_bottom(-settings.pop_offset_pct),
                    to: current_pos,
                    duration,
                    easing,
                })
            }
            StageTransitionName::None => Ok(Self::None),
        }
    }
}
