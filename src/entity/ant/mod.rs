use ggez::graphics;
use rand::{seq::SliceRandom, Rng};
use semeion::*;
use std::any::Any;

use crate::entity::phero;
use crate::{entity, game};
use memory::*;

mod memory;

/// The Ant current activity.
#[derive(Debug, PartialEq, Eq)]
enum Activity {
    /// The Ant is is search of food following
    Foraging,
    /// The Ant is carrying a portion of food back to the nest.
    Carrying,
}

/// The current state of the Ant from the point of view of the neighbor Ants.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    Leader,
    Follower,
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

pub struct Ant<'e> {
    id: entity::Id,
    location: Location,
    nest_location: Location,
    scope: Scope,
    activity: Activity,
    state: State,
    phero_concentration: phero::Concentration,
    memory: LocationAwareness,
    offspring: Offspring<'e, entity::Kind, ggez::Context>,
    context: &'e game::Context,
}

impl<'e> Ant<'e> {
    /// Constructs a new Ant located in its Nest when born.
    pub fn new(
        location: impl Into<Location>,
        context: &'e game::Context,
    ) -> Box<Self> {
        let id = context.unique_id();
        let location = location.into();
        // the Ant can only see its immediate surroundings
        let scope = Scope::with_magnitude(1);
        // the phero concentration strength left by the Ant is proportional
        // to the distance from the source (Nest/Morsel)
        let phero_concentration =
            context.conf.ants.max_phero_concentration.into();
        // all the Ants are followers at each step, until decided otherwise
        let state = State::Follower;

        Box::new(Self {
            id,
            location,
            nest_location: location,
            scope,
            activity: Activity::Foraging,
            state,
            phero_concentration,
            memory: LocationAwareness::new(context.conf.ants.memory_span),
            offspring: Offspring::default(),
            context,
        })
    }

    /// Gets the location of the first neighbor Entity of the given Kind, found
    /// in the immediate surroundings of the Ant.
    fn get_location_with_kind(
        &self,
        kind: entity::Kind,
        neighborhood: &Neighborhood<entity::Kind, ggez::Context>,
    ) -> Option<Location> {
        let border = neighborhood
            .immediate_border(self.scope)
            .expect("Invalid border");
        border
            .iter()
            .map(|tile| tile.entities().filter(|e| e.kind() == kind))
            .flatten()
            .next()
            .and_then(|e| e.location())
    }

    /// Gets the Location of the Tile within the neighborhood of this Ant, that
    /// contains the Phero entities, which scent strength sum is the greatest,
    /// unless this Ant has been there before (as far as it can remember).
    fn get_location_with_best_concentration_of(
        &self,
        scent: phero::Scent,
        neighborhood: &Neighborhood<entity::Kind, ggez::Context>,
    ) -> Option<Location> {
        let border = neighborhood
            .immediate_border(self.scope)
            .expect("Invalid border");
        border
            .iter()
            .filter(|tile| {
                // try to avoid looking in places where the Ant has been already
                // to avoid getting stuck in local maxima or minima
                !self.memory.contains(tile.location())
            })
            .filter_map(|tile| {
                let phero_kind = entity::Kind::Phero { scent };
                debug_assert!(tile.count_kind(phero_kind) <= 1);
                // get the strength of the Phero entity with the given scent
                tile.entities()
                    .find(move |e| e.kind() == phero_kind)
                    .and_then(|p| p.lifespan())
                    .and_then(|lifespan| lifespan.length())
                    .map(|strength| (tile.location(), strength))
            })
            // find the tile with the overall strongest scent
            .max_by_key(|(_, strength)| *strength)
            .map(|(location, _)| location)
    }

    /// Move towards the given kind according to the information found in the
    /// surrounding environment.
    fn move_towards(
        &mut self,
        kind: entity::Kind,
        neighborhood: &mut Neighborhood<entity::Kind, ggez::Context>,
    ) {
        // try to find a possible destination that will bring the Ant closer
        // to the destination according to its kind
        let dest = self
            // check if there is an Entity of the given kind in the neighborhood
            .get_location_with_kind(kind, neighborhood)
            // if no entity is found in the immediate surroundings, try to
            // follow the scent associated with the kind to find
            .or_else(|| {
                self.get_location_with_best_concentration_of(
                    kind.scent().expect("No scent found for kind"),
                    neighborhood,
                )
            });

        if let Some(dest) = dest {
            // follow the scent of the target pheromone
            self.location
                .translate_towards(dest, self.context.conf.env.dimension);
        } else if self.activity == Activity::Carrying
            || self.is_lost(neighborhood)
        {
            self.move_towards_nest();
        } else {
            self.move_randomly(neighborhood);
        }
    }

    /// Returns true only if this Ant cannot release more pheromone and there is
    /// no trail of pheromones in its neighborhood.
    fn is_lost(
        &self,
        neighborhood: &Neighborhood<entity::Kind, ggez::Context>,
    ) -> bool {
        self.phero_concentration.value() == 0
            && !neighborhood
                .center()
                .entities()
                .any(|e| matches!(e.kind(), entity::Kind::Phero { ..}))
    }

    /// Moves towards the nest independently of anything else, with a certain
    /// degree of accuracy, proportional to the distance from the Nest.
    fn move_towards_nest(&mut self) {
        let mut rng = rand::thread_rng();
        let dist = self
            .location
            .distance(self.nest_location, Distance::Manhattan);
        debug_assert!(dist > 0);
        let mut offsets = Offset::border(rng.gen_range(0, dist));
        debug_assert!(!offsets.is_empty());
        offsets.shuffle(&mut rng);

        let env_dimension = self.context.conf.env.dimension;
        let dest = *self
            .nest_location
            .clone()
            .translate(offsets[0], env_dimension);
        self.location.translate_towards(dest, env_dimension);
    }

    /// Moves the Ant randomly of a single tile, while trying to avoid locations
    /// that already contain the phero left that is related to the current
    /// activity.
    fn move_randomly(
        &mut self,
        neighborhood: &Neighborhood<entity::Kind, ggez::Context>,
    ) {
        // all possible neighbors offsets
        let mut offsets = Offset::border(self.scope);

        let mut rng = rand::thread_rng();
        offsets.shuffle(&mut rng);

        let offset = offsets
            .iter()
            .cloned()
            .find(|&offset| {
                let tile = neighborhood.tile(offset);
                // try to avoid looking in places where the Ant has been already
                // to avoid getting stuck in local maxima or minima
                !self.memory.contains(tile.location())
            })
            // if all the surrounding tiles cannot be avoided choose one randomly
            .unwrap_or_else(|| {
                (rng.gen_range(-1, 2), rng.gen_range(-1, 2)).into()
            });

        self.location
            .translate(offset, self.context.conf.env.dimension);
    }

    /// Leaves the pheromone according to the Ant activity and location.
    fn enhance_trail_pheromone(
        &mut self,
        neighborhood: &mut Neighborhood<entity::Kind, ggez::Context>,
    ) {
        // decrease the concentration of pheromone the Ant can leave at each
        // generation
        self.phero_concentration
            .decrease_by(self.context.conf.ants.phero_decrease);

        // check if this tile contains a pheromone entity of the same kind the
        // Ant is going to leave according to its activity
        let activity_phero_kind =
            entity::Kind::phero_with(self.activity.scent());
        let phero = neighborhood
            .center_mut()
            .entities_mut()
            .find(|e| e.kind() == activity_phero_kind);

        if let Some(phero) = phero {
            // the tile where the Ant is located already contains the pheromone
            // that it's supposed to release due to its activity -> simply increase
            // its concentration instead of releasing a new entity
            let lifespan = phero.lifespan_mut().expect("Invalid PH lifespan");
            let length = lifespan.length().expect("Invalid PH lifespan");
            let mut increase = self.phero_concentration.value() as u64;
            if self.activity.scent() == phero::Scent::Colony {
                // reinforce the path that leads to the colony nest
                increase += (length as f64
                    * self.context.conf.ants.phero_increase_ratio)
                    as u64;
            }
            lifespan.lengthen_by(increase);
        } else if self.phero_concentration.value() > 0 {
            debug_assert_eq!(neighborhood.center().location(), self.location);
            debug_assert_eq!(
                neighborhood.center().count_kind(activity_phero_kind),
                0
            );

            let is_in_community =
                neighborhood.center().count_kind(entity::Kind::Ant) > 0;
            if is_in_community {
                // we need to build consensus with the other ants that are in
                // this same tile and may release more pheromone; since we want
                // at most 1 Phero entity per tile, all the Ants in this one need
                // to decide who is going to release it
                let is_under_leader = neighborhood
                    .center()
                    .entities()
                    .filter(|e| e.kind() == entity::Kind::Ant)
                    .any(|e| {
                        let state = e
                            .state()
                            .and_then(|s| s.as_any().downcast_ref::<State>())
                            .expect("Invalid state");
                        state == &State::Leader
                    });

                if !is_under_leader {
                    self.state = State::Leader;
                }
            }

            if self.state == State::Leader || !is_in_community {
                // the tile where the Ant is located doesn't contain any pheromone
                // entity -> release a new Phero entity with a concentration
                // proportional to the distance from the source (Nest/Morsel)
                self.offspring.insert(entity::Phero::new(
                    self.activity.scent(),
                    self.location,
                    self.phero_concentration,
                    self.context,
                ));
            }
        }
    }

    /// Suppresses the pheromone found in the same Tile the Ant is located in if
    /// it's believed it may be part of a misleading trail.
    fn suppress_trail_pheromone(
        &mut self,
        neighborhood: &mut Neighborhood<entity::Kind, ggez::Context>,
    ) {
        // Try to understand if the trail of pheromones the Ant is currently in
        // leads to the target. If the target is not nearby, verify that the
        // pheromone found in this tile is not the strongest, in which case it
        // would mean the Ant may be in a misleading trail -> suppress the
        // pheromone in this tile.

        if neighborhood.contains_kind(self.activity.target_kind()) {
            return;
        }

        // check if the tile contains the pheromone entity that would lead
        // to the Ant target (Nest/Morsel) according to its activity
        let target_phero_kind =
            entity::Kind::phero_with(self.activity.target_scent());

        let border = neighborhood
            .immediate_border(self.scope)
            .expect("Invalid Border");

        let neighbor_phero_strength = border
            .iter()
            .map(|t| t.entities().filter(|e| e.kind() == target_phero_kind))
            .flatten()
            .filter_map(|e| e.lifespan().and_then(|l| l.length()))
            .max()
            .unwrap_or(0);

        let tile_phero_lifespan = neighborhood
            .center_mut()
            .entities_mut()
            .find(|e| e.kind() == target_phero_kind)
            .and_then(|e| e.lifespan_mut());

        if let Some(tile_phero_lifespan) = tile_phero_lifespan {
            let strength = tile_phero_lifespan.length().unwrap_or(0);
            if strength > neighbor_phero_strength {
                // if the highest pheromone concentration is indeed found in the
                // current tile while there is no target in the neighborhood, this
                // trail may be misleading -> clear the pheromone concentration
                tile_phero_lifespan.clear();
            }
        }
    }

    /// Checks if the Ant is located in any target (Nest or Morsel) and takes
    /// actions accordingly.
    fn assess_location_for_targets(
        &mut self,
        neighborhood: &mut Neighborhood<entity::Kind, ggez::Context>,
    ) {
        // check if the Ant is in the same location of a possible target
        for &target in &[entity::Kind::Nest, entity::Kind::Morsel] {
            let target_entity = get_overlapping_kind_mut(target, neighborhood);
            if let Some(target_entity) = target_entity {
                debug_assert_eq!(target_entity.kind(), target);
                debug_assert_eq!(self.location(), target_entity.location());

                // drop the food into the nest
                if self.activity == Activity::Carrying
                    && target == entity::Kind::Nest
                {
                    target_entity
                        .state_mut()
                        .and_then(|s| {
                            s.as_any_mut().downcast_mut::<entity::nest::State>()
                        })
                        .expect("Cannot get Nest state")
                        .store();
                }

                // if the Ant reached its target, switch its activity and reset
                // it memory
                if target == self.activity.target_kind() {
                    // there may be more than a single Ant in this Morsel and we
                    // must avoid taking more food than it actually stores
                    if target == entity::Kind::Morsel {
                        let lifespan = target_entity
                            .lifespan_mut()
                            .expect("Invalid Morsel lifespan");
                        if lifespan.is_alive() {
                            lifespan.shorten();
                            self.activity.switch();
                            self.memory.clear();
                        }
                    } else {
                        self.activity.switch();
                        self.memory.clear();
                    }
                }

                // reset the pheromone concentration
                self.phero_concentration =
                    self.context.conf.ants.max_phero_concentration.into();
            }
        }
    }
}

impl<'e> Entity<'e> for Ant<'e> {
    type Kind = entity::Kind;
    type Context = ggez::Context;

    fn id(&self) -> entity::Id {
        self.id
    }

    fn kind(&self) -> Self::Kind {
        entity::Kind::Ant
    }

    fn location(&self) -> Option<Location> {
        Some(self.location)
    }

    fn scope(&self) -> Option<Scope> {
        Some(self.scope)
    }

    fn state(&self) -> Option<&dyn entity::State> {
        Some(&self.state)
    }

    fn react(
        &mut self,
        neighborhood: Option<Neighborhood<'_, 'e, Self::Kind, Self::Context>>,
    ) -> Result<(), Error> {
        let mut neighborhood = neighborhood.expect("Invalid neighborhood");

        self.state = State::Follower;
        self.memory.insert(self.location);

        self.assess_location_for_targets(&mut neighborhood);
        self.enhance_trail_pheromone(&mut neighborhood);
        self.suppress_trail_pheromone(&mut neighborhood);
        self.move_towards(self.activity.target_kind(), &mut neighborhood);

        Ok(())
    }

    fn offspring(
        &mut self,
    ) -> Option<Offspring<'e, Self::Kind, Self::Context>> {
        // the Ant can release at most 1 Phero entity per generation
        debug_assert!(self.offspring.count() <= 1);
        Some(self.offspring.drain())
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
        let entity_size =
            entity::size(self.kind(), self.context.conf.env.tile_side);
        let center_offset = entity_size / 2.0 - env_side / 2.0;
        let loc = self.location.to_pixel_coords(env_side) - center_offset;
        // translate according to the current entity location
        transform *= Transform::translate(loc);

        graphics::push_transform(ctx, Some(transform.to_column_matrix4()));
        graphics::apply_transformations(ctx).map_err(Error::with_message)?;

        let mesh = self.context.kind_mesh(&self.kind());
        let color = match self.activity {
            Activity::Foraging => [1.0, 0.0, 0.0, 1.0],
            Activity::Carrying => [0.0, 0.0, 1.0, 1.0],
        };

        graphics::draw(
            ctx,
            mesh,
            graphics::DrawParam::default().color(color.into()),
        )
        .map_err(Error::with_message)?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).map_err(Error::with_message)
    }
}

impl Activity {
    /// Each Activity has a corresponding Phero scent that the Ant leaves
    /// on its trail.
    fn scent(&self) -> phero::Scent {
        match self {
            Activity::Carrying => phero::Scent::Food,
            Activity::Foraging => phero::Scent::Colony,
        }
    }

    /// Each Activity has a corresponding Kind as a target.
    fn target_kind(&self) -> entity::Kind {
        match self {
            Activity::Carrying => entity::Kind::Nest,
            Activity::Foraging => entity::Kind::Morsel,
        }
    }

    /// Each activity target has a Scent.
    fn target_scent(&self) -> phero::Scent {
        self.target_kind()
            .scent()
            .expect("Cannot find scent for kind")
    }

    /// Switches the Ant activity.
    fn switch(&mut self) {
        *self = match self {
            Activity::Carrying => Activity::Foraging,
            Activity::Foraging => Activity::Carrying,
        };
    }
}

/// Gets the first Entity of the given Kind that is located in the same
/// location of this Ant.
fn get_overlapping_kind_mut<'n, 'e>(
    kind: entity::Kind,
    neighborhood: &'n mut Neighborhood<'_, 'e, entity::Kind, ggez::Context>,
) -> Option<&'n mut entity::Trait<'e, entity::Kind, ggez::Context>> {
    neighborhood
        .center_mut()
        .entities_mut()
        .find(|e| e.kind() == kind)
}

/// Constructs a new mesh for an Ant.
pub fn mesh(
    ctx: &mut ggez::Context,
    conf: &game::Conf,
) -> ggez::GameResult<graphics::Mesh> {
    let color = graphics::WHITE;
    let entity_size = entity::size(entity::Kind::Ant, conf.env.tile_side);
    let tolerance = 2.0;
    let radius = entity_size / 2.0;
    let center = [radius, radius];

    let mut mesh = graphics::MeshBuilder::new();
    mesh.circle(graphics::DrawMode::fill(), center, radius, tolerance, color);
    mesh.build(ctx)
}
