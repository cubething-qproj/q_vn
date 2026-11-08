use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};

use crate::prelude::*;

#[derive(SceneComponent, Debug, PartialEq, Eq, Clone, Default)]
#[scene(CharacterNodeProps)]
#[component(on_insert=Self::log_insert, on_remove=Self::log_remove)]
pub struct CharacterNode {
    pub character: CharacterId,
}
impl CharacterNode {
    fn log_insert(_world: DeferredWorld, ctx: HookContext) {
        info!("Inserted CharacterNode (ctx = {ctx:?})");
    }

    fn log_remove(_world: DeferredWorld, ctx: HookContext) {
        info!("Removed CharacterNode (ctx = {ctx:?})");
    }
}
#[derive(Default, Clone, PartialEq)]
pub struct CharacterNodeProps {
    /// Image handle
    pub image: Handle<Image>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Starting [UiCoord], anchored to bottom-left
    pub start_coord: UiCoord,
    /// Initial [Color]. Typically used for alpha.
    pub color: Color,
}
impl CharacterNode {
    pub fn scene(props: CharacterNodeProps) -> impl Scene {
        bsn! {
            on(on_set_sprite)
            on(on_exit)

            Node {
                position_type: PositionType::Absolute,
                bottom: {props.start_coord.bottom()},
                left: {props.start_coord.left()},
            }
            Children [
                Node {
                    width: px(props.width),
                    height: px(props.height),
                    position_type: PositionType::Absolute,
                    bottom: px(0),
                    left: px(- (props.width as i32) / 2)
                }
                ImageNode {
                    image: {props.image},
                    color: {props.color},
                }
            ]
        }
    }
}
