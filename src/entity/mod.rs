use semeion::*;
use serde::{Deserialize, Serialize};

pub use ant::*;
pub use grid::*;
pub use morsel::*;
pub use nest::*;
pub use phero::*;

pub mod ant;
pub mod grid;
pub mod morsel;
pub mod nest;
pub mod phero;

/// The kinds of all the entities.
#[derive(
    Debug,
    Hash,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
    Copy,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Grid,
    Phero { scent: phero::Scent },
    Nest,
    Morsel,
    Ant,
}

impl Kind {
    /// Creates a new kind of Phero with the Scent of food.
    pub fn phero_with(scent: phero::Scent) -> Self {
        Self::Phero { scent }
    }

    /// Gets the phero Scent used to seek the Kind of self.
    pub fn scent(&self) -> Option<phero::Scent> {
        match self {
            Self::Nest => Some(phero::Scent::Colony),
            Self::Morsel => Some(phero::Scent::Food),
            _ => None,
        }
    }
}

/// Gets the size of the entity in number of pixels according to the given
/// tile side.
fn size(kind: Kind, side: f32) -> f32 {
    match kind {
        Kind::Grid => 0.0,
        Kind::Nest => side + side * 0.1,
        Kind::Morsel => side + side * 0.1,
        Kind::Phero { .. } => side,
        Kind::Ant => side - side * 0.2,
    }
}
