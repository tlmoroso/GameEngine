# GameEngine

A game engine written in Rust based on the [Coffee](https://github.com/hecrj/coffee) game engine by Hector Ramon.

## Setup

1. Follow the Rust [tutorial](https://www.rust-lang.org/learn/get-started) for setting up Rust
2. Graph generator requires python 3 and node.js. Ignore that for now though.
3. Pull the test game repo to see how a user would hook into the game engine. Also used for actual testing. (NOT ON GITHUB YET)

## Description

1. Understanding Rust: Read the [Rust Book](https://doc.rust-lang.org/book/) to get a good grasp on the Rust language
2. Important libraries to understand for working on this project: [Coffee](https://docs.rs/coffee), [Serde](https://docs.rs/serde), [Specs](https://docs.rs/specs), [Kira](https://docs.rs/kira/), [Tokio](https://docs.rs/tokio/), [thiserror](https://docs.rs/thiserror), [anyhow](https://docs.rs/anyhow), [tracing](https://docs.rs/tracing)
    - The most critical to understand are [Coffee](https://docs.rs/coffee), [Serde](https://docs.rs/serde), & [Tokio](https://docs.rs/tokio/)
3. lib.rs is the entry point to this library. It describes what modules are exposed (or not exposed) to the user.
    - Modules (mod) that are not public (pub) still must be put in lib.rs if you want them to be internally visible to other parts of the library. 
4. After that, game.rs contains the top-level data structure used by clients. We have a `struct MyGame` that implements Coffee's `Game trait` and our own `trait GameWrapper`. 
Between `MyGame` and `GameWrapper`, we force the user to comply to our form of game architecture while also allowing them some custom setup flexbility by having them implement 
the `GameWrapper trait`.
5. You'll notice in `GameWrapper`'s `load` function, along with a Specs `World`, the user also has to return a `SceneStack`. This is the next level down in the tree of data 
structures that make up a game. A `SceneStack` holds `Scenes` which can be thought of as the "levels" of a game or the different screens in a game. An inventory window or 
pause screen can be `Scenes` just like an area of the game world. They are in a stack because you want them to be... well... stackable. A pause screen is "pushed" on top of the
game, then "popped" when the user unpauses. 
6. `MyGame`, `SceneStack`, & `Scene` all have `draw`, `update`, & `interact` functions because these are the main operations of the game loop. The game loop calls down to the
top `Scene` to run the current phase plus each intermediate level does what it needs to do as well. 
7. Inside of a scene, you need the actual concrete things that make up a game: objects and actions. These are governed by the three submodules `entities`, `components`, 
& `systems`. If you already read about [Specs](https://docs.rs/specs) and the idea of ECS, then you understand how this works. Otherwise, go read a quick summary now. Entities
are purely conceptual for us. Specs takes care of the data structures for entities, and we never handle them directly unless we need specific Entity uuids for something. 
Components, however, require us (or the user) to build a struct for them to hold their state. Systems require both state and code from the user. Generally, there is just code,
but it is also possible to have state in a system. 
8. One of the main focuses of the game engine is to allow users to focus more on design than programming. One way in which we do this is by having a helpful loading system,
so most of the work can be done in JSON files instead of Rust. The tree of loading can be followed down from `MyGame` to `SceneStack` to each individual `Scene`, then through
all the entities and the components for each of those entities. You will see that each level has a corresponding Loader `struct` that can be thought of as a factory for
building the actual `struct`s such as `EntityLoader` for `Entity`. It will also have a JSON `struct` whose member variables will line up with the keys of the json dict
that will describe the object. Each layer will access the next one by building the Loaders with the file paths given in its own JSON file. 
9. Systems, for now, do not need to follow this same pattern or be loaded at all. Instead, they will be kept on the user side since they involve actual code; however, in the
future, we may provide some way for users to plug in code in higher-level languages like Python for more accessibility. This is the reason `Scene` is a trait, so users can
write the update function on their side and use their own systems. In the future, it may be useful to implement loading for systems, using `specs::Dispatcher`s possibly, to 
make it easier to plug-and-play systems into a scene. For instance, a user wants the physics system to be universally accessible, since it will be used in a lot of scenes,
but you don't need it for the pause menu scene. It would be nice to be able to specify "physics" in the list of systems for a scene in the JSON file or simply leave it out, 
so it is not running while the game is paused, and you minimize duplicated code in the implementations of `Scene` for the user. 
