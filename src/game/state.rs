use ggez::{event, graphics, mint, timer};
use rand::{rngs::StdRng, Rng, SeedableRng};
use semeion::*;
use std::{process, sync::Arc};

use crate::{entity, game};

/// The global state of the game.
pub struct State<'e> {
    /// The environment where the ant colony simulation takes place.
    pub env: Environment<'e, entity::Kind, ggez::Context>,
    /// The game context.
    context: Arc<game::Context>,
}

impl<'e> State<'e> {
    /// Constructs the game state by populating the environment with the initial
    /// entities.
    pub fn new(context: Arc<game::Context>) -> ggez::GameResult<Self> {
        let mut env = Environment::new(context.conf.env.dimension);
        debug_assert_eq!(env.dimension(), context.conf.env.dimension.into());

        // populate the environment
        env.insert(entity::Grid::new(Arc::clone(&context)));
        let nest_location = context.conf.nest.location;
        env.insert(entity::Nest::new(nest_location, Arc::clone(&context)));

        for _ in 0..context.conf.count(entity::Kind::Ant) {
            env.insert(entity::Ant::new(nest_location, Arc::clone(&context)));
        }

        let mut rng = StdRng::seed_from_u64(context.conf.seed.unwrap_or(0));
        for _ in 0..context.conf.count(entity::Kind::Morsel) {
            let location = (
                rng.gen_range(0..env.dimension().x),
                rng.gen_range(0..env.dimension().y),
            );
            env.insert(entity::Morsel::new(
                location,
                Lifespan::with_span(context.conf.morsels.storage),
                Arc::clone(&context),
            ));
        }

        Ok(Self { env, context })
    }

    /// Returns true only if the simulation is over, that is all the food has
    /// been moved from the morsels to the nest.
    pub fn is_simulation_over(&self) -> bool {
        debug_assert!(self.storage() <= self.context.conf.total_storage());
        self.storage() == self.context.conf.total_storage()
    }

    /// Gets the amount of food currently stored in the Nest.
    fn storage(&self) -> u64 {
        self.env
            .entities()
            .find(|e| e.kind() == entity::Kind::Nest)
            .and_then(|e| e.state())
            .and_then(|s| s.as_any().downcast_ref::<entity::nest::State>())
            .expect("Cannot get Nest state")
            .storage()
    }

    /// Draw simulation statistics.
    fn draw_stats(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let mut text = format!(
            "Collected: {}/{}",
            self.storage(),
            self.context.conf.total_storage()
        );
        text += &format!("\nGeneration: {}", self.env.generation());

        let foreground = graphics::Color::WHITE;
        let fragment = graphics::TextFragment::new(text).color(foreground);
        let text = graphics::Text::new(fragment);

        let dest = mint::Point2 { x: 10.0, y: 10.0 };
        graphics::draw(ctx, &text, graphics::DrawParam::default().dest(dest))?;
        Ok(())
    }
}

impl<'e> event::EventHandler<ggez::GameError> for State<'e> {
    /// Updates the game state by moving the environment forward to the next
    /// generation.
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let target_fps = self.context.conf.fps;
        let mut step = || {
            self.env
                .nextgen()
                .expect("Cannot move to the next generation");

            if self.is_simulation_over() {
                log::info!(
                    "Simulation over after {} generations",
                    self.env.generation()
                );
                process::exit(0);
            }
        };

        if let Some(fps) = target_fps {
            while timer::check_update_time(ctx, fps) {
                step();
            }
        } else {
            step();
        }

        Ok(())
    }

    /// Draws the environment with all its entities.
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, self.context.conf.env.background.into());

        self.env
            .draw(ctx, Transform::identity())
            .expect("Cannot draw the environment");

        self.draw_stats(ctx)?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }
}
