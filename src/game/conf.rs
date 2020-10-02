use anyhow::Result;
use semeion::Size;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::entity::{phero, Kind};

/// The game configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conf {
    pub fps: Option<u32>,
    pub seed: Option<u64>,
    pub env: Environment,
    pub nest: Nest,
    pub ants: Ants,
    pub morsels: Morsels,
    pub pheromones: Pheromones,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            fps: Some(24),
            seed: Some(0),
            env: Environment::default(),
            nest: Nest::default(),
            ants: Ants::default(),
            morsels: Morsels::default(),
            pheromones: Pheromones::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub dimension: (i32, i32),
    pub tile_side: f32,
    pub background: (u8, u8, u8),
    pub grid: Grid,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            dimension: (30, 30),
            tile_side: 25.0,
            background: (25, 75, 75),
            grid: Grid { visible: false },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nest {
    pub visible: bool,
    pub location: (i32, i32),
}

impl Default for Nest {
    fn default() -> Self {
        Self {
            visible: true,
            location: (25, 25),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ants {
    pub visible: bool,
    pub count: usize,
    pub memory_span: usize,
    pub max_phero_concentration: u16,
    pub phero_decrease: u16,
    pub phero_increase_ratio: f64,
}

impl Default for Ants {
    fn default() -> Self {
        Self {
            visible: true,
            count: 10,
            memory_span: 30,
            max_phero_concentration: 200,
            phero_decrease: 2,
            phero_increase_ratio: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Morsels {
    pub visible: bool,
    pub count: usize,
    pub storage: u64,
}

impl Default for Morsels {
    fn default() -> Self {
        Self {
            visible: true,
            count: 20,
            storage: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pheromones {
    pub colony: ColonyPhero,
    pub food: FoodPhero,
}

impl Default for Pheromones {
    fn default() -> Self {
        Self {
            colony: ColonyPhero { visible: false },
            food: FoodPhero { visible: false },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColonyPhero {
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodPhero {
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub visible: bool,
}

impl Conf {
    /// Parses the game configuration and returns a new Environment.
    pub fn parse(config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref();
        log::info!("Parsing game configuration from {:?}", config_path);
        let contents = fs::read_to_string(config_path)?;
        let conf = serde_json::from_str(&contents)?;
        Ok(conf)
    }

    /// Gets the size of the environment in number of pixels.
    pub fn size(&self) -> Size {
        let width = self.env.dimension.0 as f32 * self.env.tile_side;
        let height = self.env.dimension.1 as f32 * self.env.tile_side;
        (width, height).into()
    }

    /// Returns true only if the given kind should be drawn.
    pub fn is_visible(&self, kind: &Kind) -> bool {
        match kind {
            Kind::Grid => self.env.grid.visible,
            Kind::Ant => self.ants.visible,
            Kind::Morsel => self.morsels.visible,
            Kind::Nest => self.nest.visible,
            Kind::Phero { scent } => match scent {
                phero::Scent::Colony => self.pheromones.colony.visible,
                phero::Scent::Food => self.pheromones.food.visible,
            },
        }
    }

    /// Gets initial the number of entities of the given kind.
    pub fn count(&self, kind: Kind) -> usize {
        match kind {
            Kind::Ant => self.ants.count,
            Kind::Morsel => self.morsels.count,
            _ => 0,
        }
    }

    /// Gets the total food initially located in the environment.
    pub fn total_storage(&self) -> u64 {
        self.morsels.storage * self.morsels.count as u64
    }
}
