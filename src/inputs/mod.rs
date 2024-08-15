//! CLI argument types
pub mod cidr;
pub mod domain;
pub mod host;
pub mod numbers;

pub mod prelude {
    pub use super::{cidr::*, domain::*, host::*, numbers::*};
}
