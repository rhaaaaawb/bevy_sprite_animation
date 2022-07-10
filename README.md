# bevy_sprite_animation

A simple 2d sprite animation plugin for the Bevy game engine.

Anyone is welcome to make suggestion and corrections to this repository, typographic and otherwise!

This is more or less a copy of **[Aarthificial's Reanimator](https://github.com/aarthificial/reanimation)** for Unity but for Bevy of course.

**[Here](https://youtu.be/6fuo8jm7wlM)** is a video explaining how the example works.

*subject to change with feedback*

## Usage

### Add `AnimationPlugin<T>` and other systems to app

```rust
fn main() {
    App::build()
        .add_plugin(AnimationPlugin::<MainAnimation>::default())
        .add_startup_system(add_nodes.system())
        .add_startup_system(add_animator.system())
        .add_system(update_animator)
        .add_system(read_animator)
}
```

### Add nodes to `Res<NodeTree<T>>`

```rust
fn add_nodes(
    asset_server : Res<AssetServer>,
    mut node_tree : ResMut<AnimationNodes<MainAnimation>>,
) {
    // make some image handles
    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(asset_server.load(format!("SomeSprite_{}", i));
    }

    // add a node created in this system
    // hardcoded like this
    let node = Box::new(IndexNode::new("New Node", &handles));

    // with the node's id being what ever the node implements for node.id()
    // by default this is a hash_map's DefaultHasher hash of its name
    node_tree.add_node(node);

    // with a specific NodeID
    // this can be used to have multiple nodes with the same name.
    // use when loading a node if that node has a NodeID specified
    node_tree.insert_node(NodeID::from("Node Name"), node);
    
    // load a node
    // from a file
    node_tree.load("example.node");
    // from a str
    node_tree.load_node_from_str("...complex node data...");
    
    // load a node_tree
    // from a file
    node_tree.load("example.nodetree");
    // from a str
    node_tree.load_node_from_str("...any number of chained node data");
}
```

### Create an entity with an `AnimationState` on it that uses `AnimationNodes<T>` to pick its next frame

```rust
fn add_animator(
    mut commands: Commands,
) {
    // create a default state
    let mut state = AnimationState::default();
    // set starting Attributes
    start.set_attribute(Attribute::FLIP_X, true);
    // you can use custom Attributes
    // attributes can be any type that implments serde::serialize and serde::deserializeOwned
    start.set_attribute(Attribute::from_str("custom_attribute"), "cat");
    // if you use a custom attribute the name will be stored for debugging and serialization
    start.set_attribute(Attribute::new_attribute("specil_attribute"), 5);

    // set temporary attribute
    // these will be removed if they are not changed each frame
    state.set_temporary(Attribute::from_str("Index(Idel)"));

    // remove temporary attribute
    // by default all attributes are persistent
    state.set_persistent(Attribute::from_str("Index(Idel)"));

    // add a sprite bundle
    commands.spawn_bundle(SpriteBundle::default())
    // add the state
    .insert(state)
    // add the flag for the AnimationNodes<T> to use
    .insert(MainAnimation)
    // add a start node which can be the node's name or the id
    .insert(StartNode::from_str("0x1"));
}
```

### Change the state of the `AnimationState` to control what frame is picked next update

```rust
fn update_animation_state(
    mut animatiors : Query<&mut AnimationState>,
    input : Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space){
    for mut animatior in animatiors.iter(){
      start.set_attribute(Attributes::from_str("custom_attribute"), "dog");
    }}
}
```

### Get an attribute from an `AnimationState` to create logic that happens only on special frames

```rust
fn read_animation_state(
    animatiors : Query<(Entity, &AnimationState)>,
) {
    for (entity, animatior) in animatiors.iter(){
      if let Ok(ground_type) = animatior.get_attribute::<GroundType>(Attributes::from_str("step")) {
        println!("{} is on a frame where you should play the sound of someone stepping on {}", entity, ground_type);
      }
    }
}
```

### Check if an attribute from `AnimationState` changed this frame

```rust
fn read_animation_change(
    animatiors : Query<(Entity, &AnimationState)>,
    dogs: Query<&mut Dogs>,
) {
    for (entity, animatior) in animatiors.iter(){
        // assuming barke is temporary it will only change when set to true.
        // use `changed` for logic where you dont care what the attribute
        if animatior.changed(Attributes::from_str("barke")) {
            println!("{} is on a frame where you should play a barke sound effect", entity);
        }
    }

    for (entity, animatior) in animatiors.iter(){
        if animatior.changed(Attributes::from_str("dog_breed")) {
            let dog = dogs.get(animatior.get_attribute::<Entity>(Attributes::from_str("dog_breed")));
            // do something to the state based on the dog's breed
            println!("{} is on a frame where you should play a barke sound effect", entity);
        }
    }
}
```