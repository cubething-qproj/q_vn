use std::{collections::VecDeque, str::FromStr};

use bevy::text::{LineHeight, TextLayoutInfo};

use crate::{
    prelude::*,
    view::{
        settings::{DelayKind, TypewriterSettings},
        ui::interaction::DialogueState,
    },
};

#[derive(EntityEvent, Debug, Clone)]
pub struct StartTypewriterEvent {
    pub entity: Entity,
    pub typewriter: Typewriter,
}
#[derive(EntityEvent, Debug, Clone)]
pub struct TypewriterFinishedEvent {
    pub entity: Entity,
}
#[derive(EntityEvent, Debug, Clone)]
pub struct CancelTypewriterEvent {
    pub entity: Entity,
}

/// Emitted by [`await_pagination`] once a [`PaginationProbe`] has been
/// shaped and split into pages. Consumed by [`on_pagination_ready`].
#[derive(EntityEvent, Debug, Clone)]
pub struct PaginationReadyEvent {
    /// The [`DialogueBox`] the pages belong to.
    pub entity: Entity,
    /// The paginated text, one entry per page.
    pub pages: Vec<String>,
    /// Typewriter config carried through from the original
    /// [`StartTypewriterEvent`].
    pub typewriter: Typewriter,
}

/// Marker inserted by [`on_typewriter`] and consumed by
/// [`spawn_pagination_probes`]. Holds the typewriter config until the
/// target [`DialogueTextBox`] has a populated [`ComputedNode`] (i.e. UI
/// layout has run at least once for it), at which point a
/// [`PaginationProbe`] can be spawned at the correct width.
#[derive(Component, Debug, Clone)]
pub(crate) struct PendingTypewriter {
    typewriter: Typewriter,
}

/// Hidden probe entity spawned by [`spawn_pagination_probes`]. Its `Text`
/// is shaped at the target [`DialogueTextBox`] width;
/// [`await_pagination`] reads its [`TextLayoutInfo`] once populated,
/// splits the source into pages, and despawns the probe.
#[derive(Component, Debug)]
pub(crate) struct PaginationProbe {
    /// The [`DialogueBox`] being paginated for.
    for_box: Entity,
    /// Typewriter config to carry through to [`PaginationReadyEvent`].
    typewriter: Typewriter,
    /// Height budget: how many shaped lines fit on a page.
    lines_per_page: usize,
}

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct TypewriterDelay(pub Timer);

/// Indicates that this [DialogueBox] is currently typing out its contents.
#[derive(Component, Debug, Clone)]
pub struct Typewriter {
    /// The remaining un-typed text on the current page.
    text: String,
    /// Pages queued after the current one, in order.
    pages: VecDeque<String>,
    /// True once `text` has been fully typed for the current page.
    finished_current_page: bool,
}
impl Typewriter {
    /// Create a new [Typewriter].
    pub fn new(text: String) -> Self {
        Self {
            text,
            pages: VecDeque::new(),
            finished_current_page: false,
        }
    }
    /// The current page is done typing but more pages remain.
    pub fn is_awaiting_continue(&self) -> bool {
        self.finished_current_page && !self.pages.is_empty()
    }
    /// All pages have been fully typed out.
    pub fn is_fully_done(&self) -> bool {
        self.finished_current_page && self.pages.is_empty()
    }
    /// Advance to the next page. Caller is responsible for clearing the
    /// visible text node first.
    pub fn advance_page(&mut self) {
        self.text = self.pages.pop_front().unwrap_or_default();
        self.finished_current_page = false;
    }
    /// The remaining un-typed text on the current page.
    pub fn remaining(&self) -> &str {
        &self.text
    }
    /// Skip the rest of the current page's typing animation.
    pub fn finish_current_page(&mut self) {
        self.text.clear();
        self.finished_current_page = true;
    }
}

/// How a [Typewriter] places each character
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TypewriterTransition {
    /// The characters fade in.
    Fade,
    /// No transition.
    #[default]
    None,
}

pub mod prelude {
    pub use super::{
        CancelTypewriterEvent, PaginationReadyEvent, StartTypewriterEvent, Typewriter,
        TypewriterTransition,
    };
}

/// Split `source` into pages using the already-shaped `layout`.
///
/// The layout was produced by shaping the full source at the target
/// [`DialogueTextBox`] width, so line breaks already respect whitespace,
/// kerning, and font shaping. We just need to translate glyph-line
/// boundaries back into source byte offsets.
///
/// Parley strips trailing whitespace at soft-wrap points, so glyph count
/// and source char count drift by one per shaped line-break. We walk
/// source and glyphs in lock-step and consume whitespace runs explicitly
/// whenever the glyph stream jumps to a new line, which keeps the two in
/// sync regardless of how many spaces got elided.
///
/// Still assumes 1 glyph = 1 non-whitespace `char` within a line. That's
/// true for plain ASCII English; ligatures (`fi`, `fl`), grapheme
/// clusters, and fallback fonts can break it. TODO: exact source mapping
/// via shaper cluster info once Bevy exposes it.
fn paginate(source: &str, layout: &TextLayoutInfo, lines_per_page: usize) -> Vec<String> {
    let lines_per_page = lines_per_page.max(1);
    if source.is_empty() || layout.glyphs.is_empty() {
        return vec![source.to_string()];
    }
    let max_line = layout
        .glyphs
        .iter()
        .map(|g| g.line_index as usize)
        .max()
        .unwrap_or(0);
    if max_line < lines_per_page {
        return vec![source.to_string()];
    }

    // Build a byte-offset for the start of each shaped line by walking
    // source and glyphs together. Whenever the glyph stream advances a
    // line, skip any whitespace in source before recording the new line's
    // start.
    let mut src_iter = source.char_indices().peekable();
    let mut line_start_bytes: Vec<usize> = vec![0];
    let mut cur_line: usize = 0;
    for glyph in &layout.glyphs {
        while glyph.line_index as usize != cur_line {
            while let Some(&(_, c)) = src_iter.peek() {
                if c.is_whitespace() {
                    src_iter.next();
                } else {
                    break;
                }
            }
            let start = src_iter.peek().map(|&(i, _)| i).unwrap_or(source.len());
            line_start_bytes.push(start);
            cur_line += 1;
        }
        // Consume the source char corresponding to this glyph.
        src_iter.next();
    }

    // Slice pages by grouping shaped lines in chunks of `lines_per_page`.
    let total_lines = line_start_bytes.len();
    let mut pages: Vec<String> = Vec::new();
    let mut page_num = 0;
    loop {
        let start_line = page_num * lines_per_page;
        if start_line >= total_lines {
            break;
        }
        let end_line = start_line + lines_per_page;
        let start = line_start_bytes[start_line];
        let end = line_start_bytes
            .get(end_line)
            .copied()
            .unwrap_or(source.len());
        pages.push(source[start..end].trim_end().to_string());
        page_num += 1;
    }
    if pages.is_empty() {
        pages.push(source.to_string());
    }
    pages
}

/// Resolve a [`LineHeight`] to logical pixels using the [`TextFont`]'s
/// `Px` font size for the `RelativeToFont` case. Non-`Px` font sizes fall
/// back to 16 px; the actual value only affects how many lines we budget
/// per page, not the shape of the text itself.
fn line_height_px(font: &TextFont, line_height: &LineHeight) -> f32 {
    let em = match font.font_size {
        bevy::text::FontSize::Px(px) => px,
        _ => 16.0,
    };
    match line_height {
        LineHeight::Px(l) => *l,
        LineHeight::RelativeToFont(r) => em * r,
    }
}

/// Observer: on [`StartTypewriterEvent`], record the request as a
/// [`PendingTypewriter`] on the box. Actual probe spawning happens in
/// [`spawn_pagination_probes`] once the box's UI layout has settled
/// (`ComputedNode` populated), which is not guaranteed at observer time
/// -- the dialogue box may have been spawned the same frame.
pub fn on_typewriter(
    event: On<StartTypewriterEvent>,
    mut commands: Commands,
    existing_probes: Query<(Entity, &PaginationProbe)>,
) {
    debug!("on_typewriter :: {event:#?}");
    // Drop any in-flight probes for this box so a stale pagination doesn't
    // race the new one and clobber the freshly-inserted Typewriter.
    for (probe_entity, probe) in &existing_probes {
        if probe.for_box == event.entity {
            commands.entity(probe_entity).despawn();
        }
    }
    commands
        .entity(event.entity)
        // Drop any punctuation pause carried over from the previous line's
        // last typed character; without this the next line stalls up to the
        // full punctuation delay before its first character appears.
        .remove::<TypewriterDelay>()
        .insert(PendingTypewriter {
            typewriter: event.typewriter.clone(),
        });
}

/// For each [`DialogueBox`] tagged with [`PendingTypewriter`] whose
/// [`DialogueTextBox`] child has a non-empty `ComputedNode`, spawn a
/// [`PaginationProbe`] sized to the box and drop the marker. Empty source
/// text short-circuits directly to [`PaginationReadyEvent`] without a
/// probe.
pub fn spawn_pagination_probes(
    mut commands: Commands,
    q_pending: Query<(Entity, &PendingTypewriter, &Children), With<DialogueBox>>,
    mut q_text: Query<
        (&mut Text, &ComputedNode, &TextFont, Option<&LineHeight>),
        With<DialogueTextBox>,
    >,
) {
    for (box_entity, pending, children) in &q_pending {
        let Some(child) = children.iter().find(|c| q_text.contains(*c)) else {
            continue;
        };
        let Ok((mut text, computed, font, line_height)) = q_text.get_mut(child) else {
            continue;
        };
        let inner = computed.content_box().size() * computed.inverse_scale_factor();
        if inner.x <= 0.0 || inner.y <= 0.0 {
            // Layout hasn't produced a real box yet; try again next frame.
            continue;
        }

        // Clear the visible textbox now so nothing renders until pagination
        // resolves and the typewriter starts feeding characters.
        text.clear();

        // Empty source: skip the probe entirely and hand back one empty
        // page synchronously.
        if pending.typewriter.text.is_empty() {
            commands.trigger(PaginationReadyEvent {
                entity: box_entity,
                pages: vec![String::new()],
                typewriter: pending.typewriter.clone(),
            });
            commands.entity(box_entity).remove::<PendingTypewriter>();
            continue;
        }

        let line_height = line_height.cloned().unwrap_or_default();
        let line_h = line_height_px(font, &line_height);
        let lines_per_page = (inner.y / line_h).floor().max(1.0) as usize;
        debug!(
            "paginate probe: box={box_entity:?} inner={inner:?} line_h={line_h} \
             lines_per_page={lines_per_page} scale={}",
            computed.inverse_scale_factor()
        );

        // Absolute + far off-screen + explicit width means the shaper wraps
        // exactly as the visible box would, while staying invisible. We
        // only forward `TextFont`; the target box's `TextLayout` (linebreak
        // / justify) is assumed to be the default. TODO: forward it if the
        // dialogue box ever customizes it.
        commands.spawn((
            PaginationProbe {
                for_box: box_entity,
                typewriter: pending.typewriter.clone(),
                lines_per_page,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-10_000.0),
                width: Val::Px(inner.x),
                ..default()
            },
            Visibility::Hidden,
            Text::new(pending.typewriter.text.clone()),
            font.clone(),
        ));
        commands.entity(box_entity).remove::<PendingTypewriter>();
    }
}

/// Poll [`PaginationProbe`]s for a populated [`TextLayoutInfo`], split
/// their source into pages, fire [`PaginationReadyEvent`] on the target
/// box, and despawn the probe. `Changed<TextLayoutInfo>` keeps this
/// effectively idle after the first hit per probe.
pub fn await_pagination(
    mut commands: Commands,
    probes: Query<(Entity, &PaginationProbe, &TextLayoutInfo), Changed<TextLayoutInfo>>,
) {
    for (probe_entity, probe, layout) in &probes {
        if layout.size.y <= 0.0 {
            continue;
        }
        let pages = paginate(&probe.typewriter.text, layout, probe.lines_per_page);
        debug!(
            "pagination ready: box={:?} lines={} pages={}",
            probe.for_box,
            layout
                .glyphs
                .iter()
                .map(|g| g.line_index)
                .max()
                .unwrap_or(0)
                + 1,
            pages.len()
        );
        commands.trigger(PaginationReadyEvent {
            entity: probe.for_box,
            pages,
            typewriter: probe.typewriter.clone(),
        });
        commands.entity(probe_entity).despawn();
    }
}

/// Observer: apply computed pages to the target [`DialogueBox`] by
/// inserting a fully-configured [`Typewriter`] component. From here
/// [`do_typewriter`] takes over.
pub fn on_pagination_ready(event: On<PaginationReadyEvent>, mut commands: Commands) {
    let mut iter = event.pages.iter().cloned();
    let first = iter.next().unwrap_or_default();
    let mut tw = event.typewriter.clone();
    tw.text = first;
    tw.pages = iter.collect();
    tw.finished_current_page = false;
    commands.entity(event.entity).insert(tw);
}

pub fn on_cancel_typewriter(
    event: On<CancelTypewriterEvent>,
    mut q_typewriter: Query<(&mut Typewriter, &Children), With<DialogueBox>>,
    mut q_text: Query<&mut Text, With<DialogueTextBox>>,
    probes: Query<(Entity, &PaginationProbe)>,
    mut commands: Commands,
) {
    info!("on_cancel_typewriter :: {event:#?}");
    // Cancel any pending / in-flight pagination for this box so the
    // eventual callback doesn't reinstate a Typewriter after cancel.
    commands.entity(event.entity).remove::<PendingTypewriter>();
    for (probe_entity, probe) in &probes {
        if probe.for_box == event.entity {
            commands.entity(probe_entity).despawn();
        }
    }
    let (mut typewriter, children) = r!(q_typewriter.get_mut(event.entity));
    let mut text = r!(children
        .iter()
        .find(|c| q_text.contains(*c))
        .and_then(|e| q_text.get_mut(e).ok()));
    // Dump only the remainder of the *current* page, not the whole line.
    text.push_str(&typewriter.text);
    typewriter.finish_current_page();
    if typewriter.is_fully_done() {
        commands.entity(event.entity).remove::<Typewriter>();
        commands.trigger(TypewriterFinishedEvent {
            entity: event.entity,
        });
    }
}

pub fn do_typewriter(
    mut q_typewriter: Query<
        (
            Entity,
            &mut Typewriter,
            &Children,
            Option<&mut TypewriterDelay>,
        ),
        With<DialogueBox>,
    >,
    mut q_textbox: Query<&mut Text, With<DialogueTextBox>>,
    time: Res<Time>,
    mut commands: Commands,
    settings: Res<TypewriterSettings>,
) {
    q_typewriter
        .iter_mut()
        .for_each(|(entity, mut typewriter, children, timer)| {
            if typewriter.finished_current_page {
                return;
            }
            if let Some(mut timer) = timer {
                timer.tick(time.delta());
                if timer.just_finished() {
                    commands.entity(entity).remove::<TypewriterDelay>();
                }
                return;
            }
            let child = r!(children.iter().find(|c| q_textbox.contains(*c)));
            let mut text = r!(q_textbox.get_mut(child));
            let next = typewriter.text.chars().next().unwrap_or_default();
            text.push(next);
            trace!("next letter: {next:?}");
            typewriter.text.remove(0);
            let kind = DelayKind::from_str(&next.to_string()).unwrap_or(DelayKind::Character);
            let timer = settings.get_timer(kind);
            commands.entity(entity).insert(TypewriterDelay(timer));

            if typewriter.text.is_empty() {
                typewriter.finished_current_page = true;
                if typewriter.pages.is_empty() {
                    commands.entity(entity).remove::<Typewriter>();
                    commands.trigger(TypewriterFinishedEvent { entity });
                }
                // else: wait for ContinueDialogueEvent to pop the next page.
            }
        });
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            do_typewriter.run_if(in_state(DialogueState::Active)),
            spawn_pagination_probes,
            await_pagination,
        ),
    );
}
