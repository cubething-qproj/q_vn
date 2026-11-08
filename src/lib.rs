#![doc = include_str!("../README.md")]

mod view;

pub mod prelude {
    pub use super::view::prelude::*;
    pub use bevy::*;
}
use prelude::*;
