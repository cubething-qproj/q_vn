use crate::prelude::*;

pub fn do_slide(
    mut q_transition: Query<(Entity, &mut SlideTransition, &mut Node)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    q_transition
        .iter_mut()
        .for_each(|(entity, mut slide, mut node)| {
            // update timer -------------------------------------------------------------
            if slide.timer.fraction() == 0. {
                info!("{slide:#?}");
                info!("initial pos=({:?},{:?})", node.left, node.bottom,);
            }
            slide.timer.tick(time.delta());

            // Perform the transition ---------------------------------------------------
            let percent_done = slide.timer.fraction();
            let ease = slide.easing.sample(percent_done).unwrap();
            let from_left = slide.from.left_f32();
            let from_bottom = slide.from.bottom_f32();
            let to_left = slide.to.left_f32();
            let to_bottom = slide.to.bottom_f32();
            let left = from_left + ((to_left - from_left) * ease);
            let bottom = from_bottom + ((to_bottom - from_bottom) * ease);
            node.left = percent(left);
            node.bottom = percent(bottom);
            trace!(
                "slide: <{:?}, {:?}> [{}%]",
                node.left,
                node.bottom,
                1. - percent_done
            );

            // finish up ----------------------------------------------------------------
            if slide.timer.just_finished() {
                info!("Transition finished");
                info!("pos=({:?},{:?})", node.left, node.bottom,);
                if let Some(callback) = slide.callback {
                    commands.run_system_with(callback, entity);
                }
                commands.entity(entity).remove::<SlideTransition>();
                return;
            }
        });
}

pub fn do_fade(
    mut q_fade: Query<(Entity, &mut FadeTransition, &Children)>,
    mut q_img_nodes: Query<&mut ImageNode>,
    time: Res<Time>,
    mut commands: Commands,
) {
    q_fade
        .iter_mut()
        .for_each(|(entity, mut fade, children)| {
            // setup --------------------------------------------------------------------
            let Some(mut img_node) = children
                .iter()
                .find(|c| q_img_nodes.contains(*c))
                .and_then(|e| q_img_nodes.get_mut(e).ok())
            else {
                error!(
                    "Unable to find ImageNode child for entity `{entity:?}`. Dropping transition and running callback."
                );
                commands.entity(entity).remove::<FadeTransition>();
                if let Some(callback) = fade.callback {
                    commands.run_system_with(callback, entity);
                }
                return;
            };

            // update timer -------------------------------------------------------------
            if fade.timer.fraction() == 0. {
                info!("{fade:#?}");
                info!(
                    "initial alpha={}",
                    img_node.color.alpha()
                );
            }
            fade.timer.tick(time.delta());

            // perform fade -------------------------------------------------------------
            let percent_done = fade.timer.fraction();
            let alpha = if fade.fade_in {
                percent_done
            } else {
                1. - percent_done
            };
            let alpha = fade.easing.sample(alpha).unwrap();
            img_node.color.set_alpha(alpha);
            info!("Fade transition: percent_done = {}; alpha = {}", percent_done, alpha);

            // finish up ----------------------------------------------------------------
            if fade.timer.just_finished() {
                info!(
                    "Fade finished. alpha={}",
                    img_node.color.alpha()
                );
                if let Some(callback) = fade.callback {
                    commands.run_system_with(callback, entity);
                }
                commands.entity(entity).remove::<FadeTransition>();
                return;
            }
        });
}

// pub fn transition(
//     mut q_transition: Query<(Entity, &mut StageTransition, &mut Node, &Children)>,
//     mut q_img_nodes: Query<&mut ImageNode>,
//     time: Res<Time>,
//     mut commands: Commands,
// ) {
//     q_transition
//         .iter_mut()
//         .for_each(|(entity, mut transition, mut node, children)| {
//             // setup --------------------------------------------------------------------
//             macro_rules! bail {
//                 ($reason:literal) => {
//                     error!($reason);
//                     commands.entity(entity).remove::<StageTransition>();
//                     return;
//                 };
//             }
//             let Some(img_node) = children
//                 .iter()
//                 .find(|c| q_img_nodes.contains(*c))
//                 .and_then(|e| q_img_nodes.get_mut(e).ok())
//             else {
//                 bail!(
//                     "Unable to find ImageNode child for entity `{entity:?}`. Dropping transition."
//                 );
//             };

//             // update timer -------------------------------------------------------------
//             if transition.timer.fraction() == 0. {
//                 info!("Transition started");
//                 info!(
//                     "pos=({:?},{:?}), alpha={}",
//                     node.left,
//                     node.bottom,
//                     img_node.color.alpha()
//                 );
//             }
//             transition.timer.tick(time.delta());

//             // Perform the transition ---------------------------------------------------
//             match transition.kind {
//                 StageTransitionKind::Fade { fade_in, .. } => {
//                     let percent_done = transition.timer.fraction();
//                     info!("Fade transition: percent_done = {}", percent_done);
//                     let alpha = if fade_in {
//                         percent_done
//                     } else {
//                         1. - percent_done
//                     };
//                     let new_node = img_node
//                         .clone()
//                         .with_color(img_node.color.with_alpha(alpha));
//                     commands.entity(entity).insert(new_node);
//                 }
//                 StageTransitionKind::Slide { to, from, .. } => {
//                     let percent_done = transition.timer.fraction();
//                     let ease = CubicOutCurve.sample(percent_done).unwrap();
//                     let left = from.left_f32() + ((to.left_f32() - from.left_f32()) * ease);
//                     let bottom = from.bottom_f32() + ((to.bottom_f32() - from.bottom_f32()) * ease);
//                     node.left = percent(left);
//                     node.bottom = percent(bottom);
//                     trace!(
//                         "slide: <{:?}, {:?}> [{}%]",
//                         node.left,
//                         node.bottom,
//                         1. - percent_done
//                     );
//                 }
//                 StageTransitionKind::None => {
//                     warn!(
//                         "StageTransitionKind::None set for entity {entity:?}. Removing transition."
//                     );
//                     commands.entity(entity).remove::<StageTransition>();
//                 }
//             }

//             // finish up ----------------------------------------------------------------
//             if transition.timer.just_finished() {
//                 info!("Transition finished");
//                 info!(
//                     "pos=({:?},{:?}), alpha={}",
//                     node.left,
//                     node.bottom,
//                     img_node.color.alpha()
//                 );
//                 if transition.will_exit {
//                     commands.entity(entity).despawn();
//                 } else {
//                     commands.entity(entity).remove::<StageTransition>();
//                 }
//                 return;
//             }
//         });
// }

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (do_slide, do_fade).run_if(in_state(DialogueState::Active)),
    );
}
