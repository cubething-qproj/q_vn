use bevy::text::RemSize;

use crate::view::{
    prelude::*,
    ui::typewriter::{on_cancel_typewriter, on_pagination_ready, on_typewriter},
};
use std::f32::consts::PI;

#[derive(SceneComponent, Default, Clone, Debug)]
#[require(Text)]
#[scene(DialogueTextBoxParams)]
pub struct DialogueTextBox;
#[derive(Default, Clone, Debug)]
pub struct DialogueTextBoxParams {
    pub font: Option<TextFont>,
}
impl DialogueTextBox {
    pub fn scene(params: DialogueTextBoxParams) -> impl Scene {
        let font = template(move |ctx| {
            let settings = ctx.resource::<ViewSettings>();
            let font = params.font.clone().unwrap_or(settings.default_font.clone());
            Ok(font)
        });
        bsn! {
            #DialogueTextBox
            Node {
                flex_grow: 1.,
            }
            @{font}
            RemMargin(UnitRect::axes(1., 0.5))
            RemPadding(UnitRect::new(1.))
        }
    }
}

#[derive(SceneComponent, Default, Clone, Debug)]
#[require(Text)]
#[scene(DialogueSpeakerParams)]
pub struct DialogueSpeaker;
#[derive(Default, Debug, Clone)]
pub struct DialogueSpeakerParams {
    font: Option<TextFont>,
    scale: Option<f32>,
}
impl DialogueSpeaker {
    pub fn scene(params: DialogueSpeakerParams) -> impl Scene {
        let font = params.font.clone();
        let font_component = template(move |ctx| {
            let settings = ctx.resource::<ViewSettings>();
            let font = font.clone().unwrap_or(settings.default_font.clone());
            Ok(font)
        });
        let font = params.font.clone();
        let node = template(move |ctx| {
            let settings = ctx.resource::<ViewSettings>();
            let remsize = ctx.resource::<RemSize>();
            let font = font.clone().unwrap_or(settings.default_font.clone());
            let size = font
                .font_size
                .eval(settings.logical_viewport_size, *remsize);
            let height = px(params.scale.unwrap_or(1.5) * size);
            Ok(Node {
                height,
                flex_grow: 0.,
                ..Default::default()
            })
        });
        bsn! {
            #DialogueSpeaker
            @{node}
            @{font_component}
            RemMargin(UnitRect::horizontal(0.5))
        }
    }
}

/// A basic dialogue box.
/// Maybe we extend this to accept a style as an enum field.
#[derive(SceneComponent, Default, Clone, Debug)]
#[scene(DialogueBoxParams)]
pub struct DialogueBox;

#[derive(Default, Clone, Debug)]
pub struct DialogueBoxParams {
    pub font: TextFont,
    pub name_font: TextFont,
}

impl DialogueBox {
    pub fn scene(params: DialogueBoxParams) -> impl Scene {
        bsn! {
            #DialogueBox
            on(on_typewriter)
            on(on_cancel_typewriter)
            on(on_pagination_ready)

            // Main wrapper
            Node {
                width: vw(100),
                height: vh(33),
                bottom: px(0),
                position_type: PositionType::Absolute,
                margin: px(0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
            }
            ZIndex(100)
            TextColor(Color::WHITE)
            BackgroundGradient(vec![Gradient::Linear(LinearGradient::new(
                PI,
                vec![
                    ColorStop::new(Color::NONE, percent(0)),
                    ColorStop::new(Color::BLACK, percent(100)),
                ],
            ))])

            // Text elements
            Children [
                @DialogueSpeaker {
                    @font: {params.name_font}
                }
                --
                @DialogueTextBox {
                    @font: {params.font}
                }
            ]
        }
    }
}
