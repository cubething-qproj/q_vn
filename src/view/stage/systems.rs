use bevy::{asset::LoadedFolder, ecs::system::SystemId};

use crate::prelude::*;

pub fn on_enter(
    mut reader: MessageReader<EnterMsg>,
    mut commands: Commands,
    character_sprites: Res<CharacterSprites>,
    folders: Res<Assets<LoadedFolder>>,
    images: Res<Assets<Image>>,
    settings: Res<ViewSettings>,
) {
    for msg in reader.read() {
        let Some(image) = character_sprites.get_character_sprite(&msg.who, &msg.sprite, &folders)
        else {
            error!(
                "Unable to find character sprite {}/{}. Did you register them?",
                msg.who.name, msg.sprite
            );
            return;
        };
        let img = r!(images.get(&image));
        let width = img.width();
        let height = img.height();
        let character = msg.who.clone();
        let start_coord = if matches!(msg.transition_name, StageTransitionName::Pop) {
            msg.from.to_coord().with_bottom(-settings.pop_offset_pct)
        } else {
            msg.from.to_coord()
        };
        let color = if matches!(msg.transition_name, StageTransitionName::Fade) {
            Color::WHITE.with_alpha(0.)
        } else {
            Color::WHITE
        };
        let id = commands
            .spawn_scene(bsn! {
                @CharacterNode {
                    character,
                    @image,
                    @width,
                    @height,
                    @start_coord,
                    @color
                }
            })
            .id();
        msg.transition.insert(&mut commands, id, None);
    }
}

/// TODO: Make this a bit more robust. Handle transitions.
pub fn on_exit(
    event: On<ExitEvent>,
    mut commands: Commands,
    mut callback_id: Local<Option<SystemId<In<Entity>, ()>>>,
) {
    if callback_id.is_none() {
        *callback_id = Some(
            commands.register_system(|id: In<Entity>, mut commands: Commands| {
                commands.entity(*id).despawn()
            }),
        );
    }
    event
        .transition
        .insert(&mut commands, event.entity, *callback_id);
}

pub fn on_set_focused(event: On<SetFocusedEvent>, mut commands: Commands) {
    todo!()
}

pub fn on_set_sprite(
    event: On<SetSpriteEvent>,
    q_cnode: Query<(&CharacterNode, &Children)>,
    mut q_img_nodes: Query<&mut ImageNode>,
    folders: Res<Assets<LoadedFolder>>,
    character_sprites: Res<CharacterSprites>,
) {
    let (cnode, children) = r!(q_cnode.get(event.entity));
    let child = r!(children.iter().find(|c| q_img_nodes.contains(*c)));
    let mut img_node = r!(q_img_nodes.get_mut(child).ok());
    let img = r!(character_sprites.get_character_sprite(&cnode.character, &event.sprite, &folders));
    img_node.image = img;
}

pub fn on_bg(
    mut reader: MessageReader<BgMsg>,
    mut commands: Commands,
    q_scene: Query<Entity, With<SceneNode>>,
    backgrounds: Res<Backgrounds>,
    folders: Res<Assets<LoadedFolder>>,
) {
    for msg in reader.read() {
        let scene = r!(q_scene.single());
        let bg = r!(backgrounds.get(&msg.file, &folders));
        commands.entity(scene).insert((
            ZIndex(-100),
            ImageNode {
                image: bg,
                ..Default::default()
            },
        ));
    }
}

pub fn on_fx(mut reader: MessageReader<FxMsg>) {
    for _msg in reader.read() {
        todo!()
    }
}

pub fn plugin(app: &mut App) {
    macro_rules! register {
        ($($name:ident, $handler:ident);+) => {
            $(app.add_message::<$name>().add_systems(Update, $handler);)+
        };
    }
    register!(EnterMsg, on_enter; BgMsg, on_bg; FxMsg, on_fx);
}
