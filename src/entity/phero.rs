use ggez::graphics;
use semeion::*;
use serde::{Deserialize, Serialize};

use crate::{entity, game};

/// The kinds of pheromones an Ant can leave as entity offspring.
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
pub enum Scent {
    /// The scent of the colony, used to trace the path back home.
    Colony,
    /// The scent of the food the Ant is carrying.
    Food,
}

/// The value representing the strength of the Phero Scent.
#[derive(Debug, Clone, Copy)]
pub struct Concentration(u16);

pub struct Phero<'e> {
    id: entity::Id,
    scent: Scent,
    location: Location,
    lifespan: Lifespan,
    context: &'e game::Context,
}

impl<'e> Phero<'e> {
    /// Constructs a new Phero Entity.
    pub fn new(
        scent: Scent,
        location: impl Into<Location>,
        concentration: impl Into<Concentration>,
        context: &'e game::Context,
    ) -> Box<Self> {
        let id = context.unique_id();
        let concentration = concentration.into();
        Box::new(Self {
            id,
            scent,
            location: location.into(),
            lifespan: Lifespan::with_span(concentration),
            context,
        })
    }
}

impl<'e> Entity<'e> for Phero<'e> {
    type Kind = entity::Kind;
    type Context = ggez::Context;

    fn id(&self) -> entity::Id {
        self.id
    }

    fn kind(&self) -> Self::Kind {
        entity::Kind::Phero { scent: self.scent }
    }

    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn lifespan(&self) -> Option<Lifespan> {
        // the remaining concentration it's encoded in the pheromone lifetime
        Some(self.lifespan)
    }

    fn lifespan_mut(&mut self) -> Option<&mut Lifespan> {
        Some(&mut self.lifespan)
    }

    fn react(
        &mut self,
        _: Option<Neighborhood<Self::Kind, Self::Context>>,
    ) -> Result<(), Error> {
        // age by decreasing the lifespan/concentration of the pheromone by a
        // single unit for each generation
        self.lifespan.shorten();
        Ok(())
    }

    fn draw(
        &self,
        ctx: &mut Self::Context,
        mut transform: Transform,
    ) -> Result<(), Error> {
        if !self.context.conf.is_visible(&self.kind()) {
            return Ok(());
        }

        // shift the center of the mesh to the center of the Tile
        let env_side = self.context.conf.env.tile_side;
        let entity_size = entity::size(self.kind(), env_side);
        let center_offset = entity_size / 2.0 - env_side / 2.0;
        let loc = self.location.to_pixel_coords(env_side) - center_offset;
        let translation = Transform::translate(loc);

        // scale according to a value proportional to the remaining lifespan
        // that represents the concentration left
        let lifespan = self.lifespan.length().unwrap_or(0) as f32;
        let max_concentration =
            self.context.conf.ants.max_phero_concentration as f32;
        let scale = (lifespan / max_concentration).min(0.5);
        let scale = Transform::scale_around(
            [scale, scale],
            [entity_size / 2.0, entity_size / 2.0],
        );

        transform *= translation * scale;

        graphics::push_transform(ctx, Some(transform.to_column_matrix4()));
        graphics::apply_transformations(ctx).map_err(Error::with_message)?;

        // the brighter the entity the more concentration it represents, up to
        // completely white (255, 255, 255)
        let val = (lifespan * 0.1) as u8;
        let color = graphics::Color::from_rgb(val, val, val);

        let mesh = self.context.kind_mesh(&self.kind());
        graphics::draw(ctx, mesh, graphics::DrawParam::default().color(color))
            .map_err(Error::with_message)?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).map_err(Error::with_message)
    }
}

impl From<u16> for Concentration {
    fn from(strength: u16) -> Self {
        Self(strength)
    }
}

impl From<Concentration> for Span {
    fn from(concentration: Concentration) -> Self {
        Span::with_length(concentration.0 as u64)
    }
}

impl Concentration {
    /// Decreases the concentration value by a single unit and return the new
    /// concentration.
    pub fn decrease_by(&mut self, value: u16) -> Self {
        self.0 = self.0.saturating_sub(value);
        Self(self.0)
    }

    /// Gets the raw value of the Concentration.
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Constructs a new mesh for an Phero depending on its kind.
pub fn mesh(
    scent: Scent,
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    let color = graphics::WHITE;
    let entity_size =
        entity::size(entity::Kind::phero_with(scent), conf.env.tile_side);
    let tolerance = 0.5;
    let radius = entity_size / 2.0;
    let center = [radius, radius];

    let mut mesh = graphics::MeshBuilder::new();
    mesh.circle(graphics::DrawMode::fill(), center, radius, tolerance, color);
    mesh.build(ctx)
}
