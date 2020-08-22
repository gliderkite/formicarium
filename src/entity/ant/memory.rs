use semeion::*;

/// Memory of Locations of fixed maximum space.
pub struct LocationAwareness {
    capacity: usize,
    next: usize,
    locations: Vec<Option<Location>>,
}

impl LocationAwareness {
    /// Constructs a new Memory with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            next: 0,
            locations: vec![None; capacity],
        }
    }

    /// Inserts a new Location in place of the oldest one.
    pub fn insert(&mut self, location: Location) {
        if self.capacity > 0 {
            self.locations[self.next] = Some(location);
            self.next = self.next.saturating_add(1).rem_euclid(self.capacity);
        }
    }

    /// Returns true only if the given Location is recorded in memory.
    pub fn contains(&self, location: Location) -> bool {
        self.locations.contains(&Some(location))
    }

    /// Forgets all the locations.
    pub fn clear(&mut self) {
        self.locations = vec![None; self.capacity]
    }
}
