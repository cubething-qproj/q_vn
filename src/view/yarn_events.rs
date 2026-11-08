use std::str::FromStr;

use crate::view::ui::interaction::EndDialogueEvent;

use super::character::CharacterRegistry;

use super::*;
use bevy_yarnspinner::events::*;

/// Hide the dialogue box.
/// TODO: Transitions.
pub fn dialogue_completed(
    event: On<DialogueCompleted>,
    mut commands: Commands,
    q_box: Query<Entity, With<DialogueBox>>,
) {
    trace!("dialogue_completed\n{event:#?}");
    for b in q_box {
        commands.entity(b).insert(Visibility::Hidden);
    }
    commands.trigger(EndDialogueEvent);
}

/// Show the dialogue box.
/// TODO: Transitions.
pub fn dialogue_started(
    event: On<DialogueStarted>,
    mut commands: Commands,
    q_box: Query<Entity, With<DialogueBox>>,
) {
    trace!("dialogue_started\n{event:#?}");
    for b in q_box {
        commands.entity(b).insert(Visibility::Visible);
    }
}

/// This function parses the yarn events into framework events.
pub fn execute_commands(
    event: On<ExecuteCommand>,
    mut commands: Commands,
    q_characters: Query<(Entity, &CharacterNode, &Node)>,
    res_characters: Res<CharacterRegistry>,
    settings: Res<ViewSettings>,
    window: Single<&Window>,
) {
    info!("execute_commands\n{event:#?}");
    let params = &event.command.parameters;
    let Ok(command) = StageCommand::from_str(&event.command.name) else {
        error!("Unknown stage command {}", event.command.name);
        return;
    };
    macro_rules! parse_param {
        ($name:ident, $at:literal) => {
            try_parse_param!($name, $at);
            let $name = match $name {
                Ok(val) => val.to_string(),
                Err(e) => {
                    error!(e);
                    return;
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
                Err(
                    format!(
                        "Missing or incorrect parameter '{}' in command '{}' at position {} (expected {})",
                        stringify!($name), event.command.raw, stringify!($at), stringify!($ty)
                    )
                )
            };
        };
    }
    macro_rules! find_character_id {
        ($var:ident, $name:ident) => {
            let Some($var) = res_characters.iter().find(|c| c.is(&$name)).cloned() else {
                error!("Character '{}' not found. Did you register them?", $name);
                return;
            };
        };
    }
    macro_rules! find_live_character {
        ($var:ident, $name:ident) => {
            let Some($var) = q_characters.iter().find(|c| c.1.character.is(&$name)) else {
                error!("Could not find live character `{}`", $name);
                info!(
                    "Live characters: {:?}",
                    q_characters
                        .iter()
                        .map(|c| &c.1.character.name)
                        .collect::<Vec<_>>()
                );
                return;
            };
        };
    }
    macro_rules! parse_position {
        ($var:ident, $name:ident) => {
            let Ok($var) = StagePosition::from_str(&$name) else {
                error!("Invalid stage position '$name' in {}", event.command.raw);
                return;
            };
        };
    }
    macro_rules! parse_transition {
        ($var:ident, $at:literal, $pos:ident) => {
            parse_transition!($var, $at, $pos, event.command.parameters)
        };
        ($var:ident, $at:literal, $pos:ident, $params:expr) => {
            #[allow(unused_mut)]
            let mut $var = if let Some(rest) = $params.get($at..)
                && !rest.is_empty()
            {
                match StageTransitionKind::from_yarn(&rest, $pos, command, &settings) {
                    Ok(t) => t,
                    Err(e) => {
                        error!("{e}");
                        return;
                    }
                }
            } else {
                StageTransitionKind::pop($pos, &settings)
            };
        };
    }

    match command {
        StageCommand::Enter => {
            parse_param!(character, 0);
            parse_param!(sprite, 1);
            parse_param!(position, 2);
            find_character_id!(who, character);
            parse_position!(from, position);
            let current_pos = from.to_coord();
            parse_transition!(transition, 3, current_pos);
            let transition_name =
                if let Some(YarnValue::String(val)) = event.command.parameters.get(3) {
                    let Ok(name) = StageTransitionName::from_str(val) else {
                        error!("Unknown transition name, {val:?}");
                        return;
                    };
                    name
                } else {
                    StageTransitionName::Pop
                };
            commands.write_message(EnterMsg {
                who,
                sprite: sprite.clone(),
                from,
                transition,
                transition_name,
            });
        }
        StageCommand::Exit => {
            parse_param!(name, 0);
            find_live_character!(c, name);
            let node = c.2;
            let current_pos = UiCoord::new_from_vals(node.bottom, node.left, &window);
            parse_transition!(transition, 1, current_pos);
            commands.trigger(ExitEvent {
                entity: c.0,
                transition,
            });
        }
        StageCommand::SetFocused => {
            parse_param!(name, 0);
            parse_param!(focused, 0);
            find_live_character!(char, name);
            let entity = char.0;
            commands.trigger(SetFocusedEvent {
                entity,
                focused: focused.parse::<bool>().unwrap_or_default(),
            });
        }
        StageCommand::SetSprite => {
            parse_param!(name, 0);
            parse_param!(sprite, 1);
            find_live_character!(char, name);
            let node = char.2;
            let current_pos = UiCoord::new_from_vals(node.bottom, node.left, &window);
            parse_transition!(transition, 2, current_pos);
            let entity = char.0;
            commands.trigger(SetSpriteEvent {
                entity,
                sprite: sprite.clone(),
                transition,
            })
        }
        StageCommand::Move => {
            let orig_params = params;
            let mut params = params.clone();
            parse_param!(name, 0);
            find_live_character!(char, name);
            let node = char.2;
            let current_pos = UiCoord::new_from_vals(node.bottom, node.left, &window);

            *params.get_mut(0).unwrap() = YarnValue::String("slide".to_string());
            info!("{:?}", params);
            parse_transition!(transition, 0, current_pos, params);
            let StageTransitionKind::Slide {
                from,
                to,
                duration,
                easing,
            } = transition
            else {
                error!(
                    "Could not parse move command '{}'",
                    orig_params
                        .iter()
                        .map(|yv| yv.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                return;
            };

            let entity = char.0;
            commands
                .entity(entity)
                .insert(SlideTransition::new(from, to, duration, easing, None));
        }
        StageCommand::Bg => {
            parse_param!(name, 0);
            commands.write_message(BgMsg { file: name.clone() });
        }
        StageCommand::Fx => {
            parse_param!(name, 0);
            commands.write_message(FxMsg { name: name.clone() });
        }
    }
}
pub fn line_hints(event: On<LineHints>) {
    info!("line_hints\n{event:#?}");
}
pub fn node_completed(event: On<NodeCompleted>) {
    info!("node_completed\n{event:#?}");
}
pub fn node_started(event: On<NodeStarted>) {
    info!("node_started\n{event:#?}");
}

pub fn present_line(
    event: On<PresentLine>,
    mut commands: Commands,
    q_box: Query<(Entity, &Children), With<DialogueBox>>,
    q_speaker: Query<Entity, With<DialogueSpeaker>>,
) {
    trace!("present_line\n{event:#?}");
    for (b, children) in q_box {
        let text = event.line.text_without_character_name();
        commands.trigger(StartTypewriterEvent {
            entity: b,
            typewriter: Typewriter::new(text),
        });

        let speaker_box = children.iter().find_map(|c| q_speaker.get(c).ok());
        let speaker_box = c!(speaker_box);
        let speaker_text = event.line.character_name().unwrap_or_default();
        commands
            .entity(speaker_box)
            .insert(Text(speaker_text.to_string()));
    }
}
pub fn present_options(event: On<PresentOptions>) {
    info!("present_options\n{event:#?}");
}

pub fn plugin(app: &mut App) {
    macro_rules! register_observers {
        ($($name:ident),+) => {
            $(app.add_observer($name.run_if(resource_exists::<YarnProject>));)+
        }
    }
    register_observers!(
        dialogue_completed,
        dialogue_started,
        node_completed,
        node_started,
        execute_commands,
        line_hints,
        present_line,
        present_options
    );
}
