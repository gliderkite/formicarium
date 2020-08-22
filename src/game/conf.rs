use anyhow::Result;
use ggez::graphics::Color;
use semeion::{Dimension, Location, Size};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::entity::{phero, Kind};

/// The game configuration.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conf {
    fps: Option<u32>,
    env: Environment,
    nest: Nest,
    ants: Ants,
    morsels: Morsels,
    pheromones: Pheromones,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Environment {
    dimension: (i32, i32),
    tile_side: f32,
    background: (u8, u8, u8),
    grid: Grid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Nest {
    visible: bool,
    location: (i32, i32),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ants {
    visible: bool,
    count: usize,
    memory_span: usize,
    max_phero_concentration: u16,
    phero_decrease: u16,
    phero_increase_ratio: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Morsels {
    visible: bool,
    count: usize,
    storage: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pheromones {
    colony: ColonyPhero,
    food: FoodPhero,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ColonyPhero {
    visible: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FoodPhero {
    visible: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Grid {
    visible: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            fps: Some(24),
            env: Environment {
                dimension: (30, 30),
                tile_side: 25.0,
                background: (25, 75, 75),
                grid: Grid { visible: false },
            },
            nest: Nest {
                visible: true,
                location: (25, 25),
            },
            ants: Ants {
                visible: true,
                count: 10,
                memory_span: 30,
                max_phero_concentration: 200,
                phero_decrease: 2,
                phero_increase_ratio: 0.1,
            },
            morsels: Morsels {
                visible: true,
                count: 20,
                storage: 30,
            },
            pheromones: Pheromones {
                colony: ColonyPhero { visible: false },
                food: FoodPhero { visible: false },
            },
        }
    }
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

    /// Gets the target fps.
    pub fn fps(&self) -> Option<u32> {
        self.fps
    }

    /// Gets the background color.
    pub fn background_color(&self) -> Color {
        self.env.background.into()
    }

    /// Gets the dimension of the environment.
    pub fn env_dimension(&self) -> Dimension {
        self.env.dimension.into()
    }

    /// Gets the side length of each tile in the environment.
    pub fn side(&self) -> f32 {
        self.env.tile_side
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

    /// Gets initial the food storage of each morsel.
    pub fn morsel_storage(&self) -> u64 {
        self.morsels.storage
    }

    /// Gets the total food initially located in the environment.
    pub fn total_storage(&self) -> u64 {
        self.morsels.storage * self.morsels.count as u64
    }

    /// Gets the nest location.
    pub fn nest_location(&self) -> Location {
        self.nest.location.into()
    }

    /// Gets the maximum concentration level of pheromone that can be released
    /// by an Ant.
    pub fn ant_max_phero_concentration(&self) -> phero::Concentration {
        self.ants.max_phero_concentration.into()
    }

    /// Gets the value that is subtracted to the Ant pheromone concentration at
    /// each generation.
    pub fn ant_phero_decrease(&self) -> u16 {
        self.ants.phero_decrease
    }

    /// Gets the value in percentage that is used to increase the pheromone found,
    /// when equal to the pheromone the Ant is supposed to leave when foraging.
    pub fn ant_phero_inc_ratio(&self) -> f64 {
        self.ants.phero_increase_ratio
    }

    /// Gets the ants memory span.
    pub fn ant_memory_span(&self) -> usize {
        self.ants.memory_span
    }
}
