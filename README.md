# formicarium

A basic ant colony simulation that aims to show how the collective behavior of
decentralized and self-organized artificial systems 
([swarm intelligence](https://en.wikipedia.org/wiki/Swarm_intelligence)), allows
multiple organism to perform *better* when working together to achieve the same
goal compared with a single organism, given the same set of skills and the same
environment.

`formicarium` is built on Rust in top of the graphic engine
[ggez](https://github.com/ggez/ggez) and the entity engine
[semeion](https://github.com/gliderkite/semeion).


## Overview

The simulation runs on a 2D environment of initial fixed size, which is initially
populated with the following entities:

- `Nest`: A single ant nest that is located in the environment, it represents the
    place where all the ants are born and where all the food needs to be
    collected.
- `Morsels`: One or multiple locations in the environment where a fixed amount
    of food is located.
- `Ants`: The entities that can freely move within the environment to collect the
    food from the morsels and bring it back to the nest.
- `Pheromones`: Dynamic entities that are created by the ants and used as a
    communication mechanism to allow other ants to take educated decisions.
    There are 2 types of pheromones:
    - `Colony`: used to reinforce a path that can lead to the `Nest`.
    - `Food`: used to reinforce a path that can lead to a `Morsel`.

The simulation starts with a fixed set of parameters that determine the
environment's initial configuration. The goal is for the ants to collect all the
food in the environment as fast as possible (lowest number of generations) and
bring it back to their nest. When all the food is found and collected, the
simulation is over.

The faster the ants can bring back the food the better, and more importantly, the
number of generations required with the same configuration should decrease faster
than linearly as the number of ants increases, proving that even the most basic
forms of *swarm intelligence* can give significant advantages when multiple
organisms work together.

Each of these entities is represented with basic geometric shapes when drawn
into the screen, and their color and size is used to represent concentration or
scope.


## How to run

To run the simulation all you need is `cargo` and a configuration file such as
the [conf.json](conf.json) that you can find on this project, which you are free
to edit to change the initial parameters.

```bash
RUST_LOG=formicarium=info cargo run --release -- conf.json
```

<img src="assets/ants.gif" width="350" height="350">

From the animation above you can see how the ants, represented as dots (red when
foraging and blue when carrying) move within the environment by following the
trail of food pheromone, represented as smaller dots (that go from black to white
according to their concentration), in order to find the morsels (the green
squares) and bring it back to the nest (the blue square).

When the trail of pheromone becomes more and more defined, more ants start
converging over a single path, increasing the fraction of food that is brought
back to the nest per unit of time/generation.

Increasing the number of ants and morsels can lead to very interesting and intricate
patterns.

<img src="assets/ants-huge.gif" width="500" height="500">

The speed of the simulation can be affected by changing the `fps` parameter in
the configuration file (which if not specified leads to maximum performance).


## The Ants

The idea behind the simulation is that all the ants have a determined set of skills,
and they can make decisions that will affect their movement according to the
portion of the environment they can interact with.

Each ant can either be looking for food (`foraging`) or, having already collected
the food from a morsel, be bringing the food back to the nest (`carrying`).
According to the current ant task its goal is to find a specific target in the
environment:
- `Foraging` targets any `Morsel`.
- `Carrying` targets the `Nest`.

Once the ant reaches its target, it simply switch its task.

In order to find their target, each ant can rely on the information retrieved
by the portion of the environment that immediately surrounds the ant, and in
particular, each ant can take its own decisions according to the entities that
currently populate the visible portion of the environment.

The main idea is that the ant can either reinforce the information found, by
trusting the trail of pheromones left previously by other ants (or itself) in
order to find its target, or cancel it out by rejecting the information found
because it's misleading and thought to possibly lead to a dead end.

Ants cannot communicate directly between each other, therefore this information
can only be exchanged by leaving pheromone entities in the environment, where
each pheromone has an intrinsic property of concentration that can be increased
or reduced by the ants (and decreases over time to simulate evaporation).
The stronger the pheromone concentration, the higher the chance to be in 
proximity of a target.

When no information is available in the surrounding environment, the ant can
still rely on a sense of direction, by knowing the approximate direction of its
nest. Moreover, each ant has the ability to remember a configurable number of
previous steps (representing its short-term memory), which allows the ant to not
end up in local minima or maxima by returning to locations that were thought to
be close to its target.

If no educated action can be taken, the ant will simply try a random step.

Given this behavior, the higher the number of ants, the stronger the pheromone
trails will be reinforced (or weakened), giving numerous groups of ants
advantages over smaller ones.


## The event loop

What follows is a minimal representation of the event loop on top of which the
simulation can run (without being drawn into the screen):


```rust
use formicarium::*;

let conf = game::Conf::parse("conf.json").unwrap();
let context = game::Context::new(conf);
let mut state = game::State::new(&context).unwrap();

while !state.is_simulation_over() {
    let generation = state.env.nextgen().unwrap();
    if generation > MAX_GENERATIONS_COUNT {
        panic!("Timeout!");
    }
}

println!("Simulation over after {} generations", state.env.generation());
```
