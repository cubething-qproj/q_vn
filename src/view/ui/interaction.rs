use bevy_yarnspinner::prelude::DialogueRunner;

use crate::prelude::*;
use crate::view::ui::typewriter::TypewriterDelay;

/// Whether a dialogue is currently running.
#[derive(States, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub enum DialogueState {
    Active,
    #[default]
    Inactive,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct StartDialogueEvent {
    pub node: String,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct ContinueDialogueEvent;

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct EndDialogueEvent;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogueSystems;

fn check_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    // TODO: Check if UI is focused, and not hovering over a button.
    if mouse.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || touches.any_just_pressed()
    {
        commands.trigger(ContinueDialogueEvent)
    }
}

fn start_dialogue(
    event: On<StartDialogueEvent>,
    mut q_runner: Query<&mut DialogueRunner>,
    mut next_state: ResMut<NextState<DialogueState>>,
) {
    debug!("start_dialogue {event:#?}");
    let mut runner = r!(q_runner.single_mut());
    runner.start_node(&event.node);
    next_state.set(DialogueState::Active)
}

fn continue_dialogue(
    event: On<ContinueDialogueEvent>,
    mut commands: Commands,
    mut q_runner: Query<&mut DialogueRunner>,
    mut q_box: Query<(Entity, &Children, &mut Typewriter), With<DialogueBox>>,
    mut q_text: Query<&mut Text, With<DialogueTextBox>>,
) {
    debug!("continue_dialogue {event:#?}");
    // If a typewriter is active, a click first services it (skip current
    // page, advance to next page, or drop it) before touching the runner.
    if let Ok((entity, children, mut tw)) = q_box.single_mut() {
        let text_child = children.iter().find(|c| q_text.contains(*c));
        if !tw.is_awaiting_continue() && !tw.is_fully_done() {
            // Mid-typing: dump the rest of the current page.
            if let Some(mut text) = text_child.and_then(|e| q_text.get_mut(e).ok()) {
                text.push_str(tw.remaining());
            }
            tw.finish_current_page();
            return;
        }
        if tw.is_awaiting_continue() {
            // Between pages: clear the box and pop the next page.
            if let Some(mut text) = text_child.and_then(|e| q_text.get_mut(e).ok()) {
                text.clear();
            }
            tw.advance_page();
            // Same rationale as in `on_typewriter`: don't inherit the
            // previous page's terminal punctuation pause.
            commands.entity(entity).remove::<TypewriterDelay>();
            return;
        }
        // Fully done: retire the typewriter and fall through to the runner.
        commands.entity(entity).remove::<Typewriter>();
    }
    let mut runner = r!(q_runner.single_mut());
    runner.continue_in_next_update();
}

fn end_dialogue(
    event: On<EndDialogueEvent>,
    mut q_runner: Query<&mut DialogueRunner>,
    mut next_state: ResMut<NextState<DialogueState>>,
) {
    debug!("end_dialogue {event:#?}");
    let mut runner = r!(q_runner.single_mut());
    runner.stop();
    next_state.set(DialogueState::Inactive)
}

pub fn plugin(app: &mut App) {
    app.init_state::<DialogueState>();
    app.add_observer(start_dialogue.run_if(in_state(DialogueState::Inactive)));
    app.add_observer(continue_dialogue.run_if(in_state(DialogueState::Active)));
    app.add_observer(end_dialogue.run_if(in_state(DialogueState::Active)));
    app.add_systems(
        Update,
        check_input
            .run_if(in_state(DialogueState::Active))
            .in_set(DialogueSystems),
    );
}
