use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    text::{EmSize, RemSize},
};

pub use crate::view::prelude::*;

//-----------------------------------------------
// CONSTS AND DATA
// ----------------------------------------------

/// A rectangle which can be applied to any unit.
/// For example, [RemMargin] uses [UnitRect] to specify
/// the margin in rem units.
#[derive(Default, Debug, Clone, Copy)]
pub struct UnitRect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}
impl UnitRect {
    pub fn left(rem: f32) -> Self {
        Self {
            left: rem,
            ..Default::default()
        }
    }
    pub fn right(rem: f32) -> Self {
        Self {
            right: rem,
            ..Default::default()
        }
    }
    pub fn top(rem: f32) -> Self {
        Self {
            top: rem,
            ..Default::default()
        }
    }
    pub fn bottom(rem: f32) -> Self {
        Self {
            bottom: rem,
            ..Default::default()
        }
    }
    pub fn new(rem: f32) -> Self {
        Self {
            left: rem,
            right: rem,
            top: rem,
            bottom: rem,
        }
    }
    pub fn horizontal(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            ..Default::default()
        }
    }
    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            ..Default::default()
        }
    }
    pub fn axes(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Takes the font size (in pixels) and calculates the UiRect in pixels.
    pub fn to_rem(self, rem_size: RemSize) -> UiRect {
        UiRect {
            left: px(self.left * *rem_size),
            right: px(self.right * *rem_size),
            top: px(self.top * *rem_size),
            bottom: px(self.bottom * *rem_size),
        }
    }
}

//-----------------------------------------------
// COMMON UI COMPONENTS
// ----------------------------------------------

/// Transient component which patches the [Node]'s padding.
#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
#[require(Node)]
#[component(on_insert=Self::resolve)]
pub struct RemPadding(pub UnitRect);
impl RemPadding {
    pub fn resolve(mut world: DeferredWorld, ctx: HookContext) {
        {
            let remsize = *world.get_resource::<RemSize>().unwrap();
            let padding = *world.get::<Self>(ctx.entity).unwrap();
            let mut node = world.get_mut::<Node>(ctx.entity).unwrap();
            node.padding = padding.to_rem(remsize);
        }
        {
            let entity = ctx.entity;
            let mut commands = world.commands();
            commands.entity(entity).remove::<Self>();
        }
    }
}

/// Transient component which patches the [Node]'s margin.
#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
#[require(Node)]
#[component(on_insert=Self::resolve)]
pub struct RemMargin(pub UnitRect);
impl RemMargin {
    pub fn resolve(mut world: DeferredWorld, ctx: HookContext) {
        {
            let remsize = *world.get_resource::<RemSize>().unwrap();
            let margin = *world.get::<Self>(ctx.entity).unwrap();
            let mut node = world.get_mut::<Node>(ctx.entity).unwrap();
            node.margin = margin.to_rem(remsize);
        }
        {
            let entity = ctx.entity;
            let mut commands = world.commands();
            commands.entity(entity).remove::<Self>();
        }
    }
}

// ----------------------------------------------
// UI UTILS
// ----------------------------------------------

fn to_percent(val: Val, window: &Window, is_x: bool) -> Val {
    let base_value = if is_x {
        window.width()
    } else {
        window.height()
    };
    let val = val.resolve(
        window.scale_factor(),
        base_value,
        window.physical_size().as_vec2(),
        EmSize::default(),
        RemSize::default(),
    );
    let val = 100. * val.unwrap_or_default() / base_value;
    percent(val)
}

/// Not to be confused with [UiPosition]
#[derive(Reflect, Debug, Default, Copy, Clone, PartialEq)]
pub struct UiCoord {
    bottom: Val,
    left: Val,
}
impl UiCoord {
    /// Takes a position in percentage.
    pub fn new(bottom: f32, left: f32) -> Self {
        Self {
            bottom: percent(bottom),
            left: percent(left),
        }
    }

    /// convert to percentage by resolving then un-resolving
    /// abs = pct * physical_base / 100    =>    pct = 100*abs/physical_base
    pub fn new_from_vals(bottom: Val, left: Val, window: &Window) -> Self {
        let bottom = if matches!(bottom, Val::Percent(_)) {
            bottom
        } else {
            to_percent(bottom, window, true)
        };
        let left = if matches!(left, Val::Percent(_)) {
            left
        } else {
            to_percent(left, window, false)
        };
        Self { bottom, left }
    }

    pub fn bottom(&self) -> Val {
        self.bottom
    }
    pub fn with_bottom(self, bottom: f32) -> Self {
        Self {
            bottom: percent(bottom),
            left: self.left,
        }
    }
    pub fn left(&self) -> Val {
        self.left
    }
    pub fn with_left(self, left: f32) -> Self {
        Self {
            bottom: self.bottom,
            left: percent(left),
        }
    }
    pub fn bottom_f32(&self) -> f32 {
        let Val::Percent(ret) = self.bottom else {
            unreachable!()
        };
        ret
    }
    pub fn left_f32(&self) -> f32 {
        let Val::Percent(ret) = self.left else {
            unreachable!()
        };
        ret
    }
}
