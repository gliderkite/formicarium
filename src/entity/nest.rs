use ggez::graphics;
use semeion::*;
use std::any::Any;

use crate::{entity, game};

/// The current state of the Ant from the point of view of the neighbor Ants.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
pub struct State {
    storage: u64,
}

/// Implement the entity::State trait to allow downcasting when querying the
/// Cell state via the Entity::state() method.
impl entity::State for State {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A static nest.
pub struct Nest<'e> {
    id: entity::Id,
    location: Location,
    context: &'e game::Context,
    state: State,
}

impl<'e> Nest<'e> {
    /// Constructs a new Nest.
    pub fn new(
        location: impl Into<Location>,
        context: &'e game::Context,
    ) -> Self {
        let id = context.unique_id();
        // the storage of food is initially empty
        let state = State::default();
        Self {
            id,
            location: location.into(),
            context,
            state,
        }
    }
}

impl<'e> Entity<'e> for Nest<'e> {
    type Kind = entity::Kind;
    type Context = ggez::Context;

    fn id(&self) -> entity::Id {
        self.id
    }

    fn kind(&self) -> Self::Kind {
        entity::Kind::Nest
    }

    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn state(&self) -> Option<&dyn entity::State> {
        Some(&self.state)
    }

    fn state_mut(&mut self) -> Option<&mut dyn entity::State> {
        Some(&mut self.state)
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
        let env_side = self.context.conf.side();
        let entity_size = entity::size(self.kind(), self.context.conf.side());
        let center_offset = entity_size / 2.0 - env_side / 2.0;
        let loc = self.location.to_pixel_coords(env_side) - center_offset;
        // translate according to the current entity location
        let translation = Transform::translate(loc);

        transform *= translation;

        graphics::push_transform(ctx, Some(transform.to_column_matrix4()));
        graphics::apply_transformations(ctx).map_err(Error::with_message)?;

        let mesh = self.context.kind_mesh(&self.kind());
        graphics::draw(ctx, mesh, graphics::DrawParam::default())
            .map_err(Error::with_message)?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).map_err(Error::with_message)
    }
}

impl State {
    /// Increment the food storage by a single unit.
    pub fn store(&mut self) {
        self.storage = self.storage.saturating_add(1);
    }

    /// Gets the total amount of food stored.
    pub fn storage(&self) -> u64 {
        self.storage
    }
}

/// Constructs a new mesh for a Nest.
pub fn mesh(
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    let mut mesh = graphics::MeshBuilder::new();
    let color = graphics::Color::new(0.1, 0.3, 0.9, 1.0);

    let entity_size = entity::size(entity::Kind::Nest, conf.side());
    let outer = graphics::Rect::new(0.0, 0.0, entity_size, entity_size);
    mesh.rectangle(graphics::DrawMode::stroke(3.0), outer, color);

    let half_size = entity_size / 2.0;
    let inner = graphics::Rect::new(
        half_size / 2.0,
        half_size / 2.0,
        half_size,
        half_size,
    );
    mesh.rectangle(graphics::DrawMode::fill(), inner, color);

    mesh.build(ctx)
}
