use ggez::graphics;
use semeion::*;

use crate::{entity, game};

/// A static grid of squared cells.
pub struct Grid<'e> {
    id: entity::Id,
    context: &'e game::Context,
}

impl<'e> Grid<'e> {
    /// Constructs a new Grid.
    pub fn new(context: &'e game::Context) -> Box<Self> {
        let id = context.unique_id();
        Box::new(Self { id, context })
    }
}

impl<'e> Entity<'e> for Grid<'e> {
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

        graphics::push_transform(ctx, Some(transform.to_column_matrix4()));
        graphics::apply_transformations(ctx).map_err(Error::with_message)?;

        let mesh = self.context.kind_mesh(&self.kind());
        graphics::draw(ctx, mesh, graphics::DrawParam::default())
            .map_err(Error::with_message)?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).map_err(Error::with_message)
    }
}

/// Constructs a new mesh for a Grid.
pub fn mesh(
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    use ggez::nalgebra::Point2;

    let mut mesh = graphics::MeshBuilder::new();
    let size = conf.size();
    let stroke_width = 2.0;
    let color = graphics::BLACK;

    // horizontal lines
    for i in 0..=conf.env_dimension().y {
        let y = i as f32 * conf.side();
        let points = [Point2::new(0.0, y), Point2::new(size.width, y)];
        mesh.line(&points, stroke_width, color)?;
    }
    // vertical lines
    for i in 0..=conf.env_dimension().x {
        let x = i as f32 * conf.side();
        let points = [Point2::new(x, 0.0), Point2::new(x, size.height)];
        mesh.line(&points, stroke_width, color)?;
    }

    mesh.build(ctx)
}
