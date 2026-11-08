use crate::prelude::*;

/// The parent node for the entire dialogue scene to render.
#[derive(SceneComponent, Debug, Clone, PartialEq, Default)]
pub struct SceneNode;
impl SceneNode {
    pub fn scene() -> impl Scene {
        bsn! {
            Node {
                width: vw(100),
                height: vh(100),
            }
        }
    }
}
