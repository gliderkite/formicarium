use ggez::graphics;
use semeion::*;
use std::sync::Arc;

use crate::{entity, game};

/// A static grid of squared cells.
pub struct Grid {
    id: entity::Id,
    context: Arc<game::Context>,
}

impl Grid {
    /// Constructs a new Grid.
    pub fn new(context: Arc<game::Context>) -> Box<Self> {
        let id = context.unique_id();
        Box::new(Self { id, context })
    }
}

impl<'e> Entity<'e> for Grid {
    type Kind = entity::Kind;
    type Context = ggez::Context;

    fn id(&self) -> entity::Id {
        self.id
    }

    fn kind(&self) -> Self::Kind {
        entity::Kind::Grid
    }

    fn draw(
        &self,
        ctx: &mut Self::Context,
        transform: Transform,
    ) -> Result<(), Error> {
        if !self.context.conf.is_visible(&self.kind()) {
            return Ok(());
        }

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

/// Constructs a new mesh for a Grid.
pub fn mesh(
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    use ggez::mint::Point2;

    let mut mesh = graphics::MeshBuilder::new();
    let size = conf.size();
    let stroke_width = 2.0;
    let color = graphics::Color::BLACK;
    let dimension: Dimension = conf.env.dimension.into();

    // horizontal lines
    for i in 0..=dimension.y {
        let y = i as f32 * conf.env.tile_side;
        let points = [Point2 { x: 0.0, y }, Point2 { x: size.width, y }];
        mesh.line(&points, stroke_width, color)?;
    }
    // vertical lines
    for i in 0..=dimension.x {
        let x = i as f32 * conf.env.tile_side;
        let points = [Point2 { x, y: 0.0 }, Point2 { x, y: size.height }];
        mesh.line(&points, stroke_width, color)?;
    }

    mesh.build(ctx)
}
