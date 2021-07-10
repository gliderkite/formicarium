use ggez::graphics;
use semeion::*;
use std::sync::Arc;

use crate::{entity, game};

/// A static morsel.
pub struct Morsel {
    id: entity::Id,
    location: Location,
    lifespan: Lifespan,
    context: Arc<game::Context>,
}

impl Morsel {
    /// Constructs a new Morsel.
    pub fn new(
        location: impl Into<Location>,
        lifespan: impl Into<Lifespan>,
        context: Arc<game::Context>,
    ) -> Box<Self> {
        let id = context.unique_id();
        Box::new(Self {
            id,
            location: location.into(),
            lifespan: lifespan.into(),
            context,
        })
    }
}

impl<'e> Entity<'e> for Morsel {
    type Kind = entity::Kind;
    type Context = ggez::Context;

    fn id(&self) -> entity::Id {
        self.id
    }

    fn kind(&self) -> Self::Kind {
        entity::Kind::Morsel
    }

    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn lifespan(&self) -> Option<Lifespan> {
        Some(self.lifespan)
    }

    fn lifespan_mut(&mut self) -> Option<&mut Lifespan> {
        Some(&mut self.lifespan)
    }

    fn draw(
        &self,
        ctx: &mut Self::Context,
        mut transform: Transform,
    ) -> Result<(), Error> {
        if !self.context.conf.is_visible(&self.kind()) {
            return Ok(());
        }

        // shift the center of the Rect to the center of the Tile
        let env_side = self.context.conf.env.tile_side;
        let entity_size =
            entity::size(self.kind(), self.context.conf.env.tile_side);
        let center_offset = entity_size / 2.0 - env_side / 2.0;
        let loc = self.location.to_pixel_coords(env_side) - center_offset;
        // translate according to the current entity location
        let translation = Transform::translate(loc);

        // scale according to a value proportional to the remaining lifespan
        let lifespan = self.lifespan.length().unwrap_or(0) as f32;
        let max_concentration = self.context.conf.morsels.storage as f32;
        let scale = (lifespan / max_concentration).min(1.0);
        let scale = Transform::scale_around(
            [scale, scale],
            [entity_size / 2.0, entity_size / 2.0],
        );

        transform *= translation * scale;

        let mesh = self.context.kind_mesh(&self.kind());
        graphics::draw(
            ctx,
            mesh,
            graphics::DrawParam::default()
                .transform(transform.to_column_matrix4()),
        )
        .map_err(Error::with_message)
    }
}

/// Constructs a new mesh for a Morsel.
pub fn mesh(
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    let mut mesh = graphics::MeshBuilder::new();
    let color = graphics::Color::new(0.3, 0.5, 0.0, 1.0);
    let entity_size = entity::size(entity::Kind::Morsel, conf.env.tile_side);

    let outer = graphics::Rect::new(0.0, 0.0, entity_size, entity_size);
    mesh.rectangle(graphics::DrawMode::stroke(1.0), outer, color)?;

    let half_size = entity_size / 2.0;
    let inner = graphics::Rect::new(
        half_size / 2.0,
        half_size / 2.0,
        half_size,
        half_size,
    );
    mesh.rectangle(graphics::DrawMode::fill(), inner, color)?;

    mesh.build(ctx)
}
