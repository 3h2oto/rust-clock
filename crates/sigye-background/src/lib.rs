//! Background animation rendering for the sigye clock.
//!
//! This crate provides animated background effects for the terminal clock,
//! including both stateless animations (computed from position/time) and
//! stateful animations (matrix rain, snowfall) as well as reactive
//! backgrounds that respond to system metrics.

mod animations;
mod chars;
mod color;
mod state;

pub use color::{hsl_to_rgb, resource_to_color};
pub use state::BackgroundState;
