use super::prelude::*;
use super::error::BevySpriteAnimationError as Error;

use serde::{Serialize, de::DeserializeOwned};

use bevy::prelude::*;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Component)]
pub struct AnimationState {
    data: HashMap<Attributes,Vec<u8>>,
    pub(crate) changed: HashSet<Attributes>,
    pub(crate) temp: HashSet<Attributes>,
}

impl Default for AnimationState {
    fn default() -> Self {
        let mut data = HashMap::default();
        data.insert(Attributes::DELTA, bincode::serialize(&0.0f32).unwrap());
        data.insert(Attributes::FRAMES, bincode::serialize(&0usize).unwrap());
        Self { data, changed: HashSet::new(), temp: HashSet::new() }
    }
}

impl AnimationState {
    /// will return `D` for attribute panics if the attribute is not set
    /// or `D` is the wrong type
    /// use try_get_attribute() if you are unsure if the attribute exists
    #[inline(always)]
    pub fn get_attribute<D: DeserializeOwned>(&self, key: Attributes) -> D {
        self.try_get_attribute(key).expect("Attribute Exists")
    }

    /// will return an `option<D>` attribute panics if `D` is the wrong type
    #[inline(always)]
    pub fn try_get_attribute<D: DeserializeOwned>(&self, key: Attributes) -> Option<D> {
        match self.try_get_attribute_or_error(key) {
            Ok(res) => Some(res),
            Err(e) => match e {
                BevySpriteAnimationError::AttributeNotFound(_) => None,
                BevySpriteAnimationError::BincodeError(e) => panic!("Attribute could not be deserialised: {}", e),
                _ => panic!("How did you get this error; please file bug report"),
            }
        }
    }

    pub(crate) fn try_get_attribute_or_error<D: DeserializeOwned>(&self, key: Attributes) -> Result<D, Error> {
        match self.data.get(&key) {
            Some(att) => {Ok(bincode::deserialize(att)?)},
            None => Err(Error::AttributeNotFound(key.clone()))
        }
    }

    pub fn set_attribute<D: Serialize>(&mut self, key: Attributes, val: D) {
        match bincode::serialize(&val) {
            Ok(v) => {
                //todo make return something
                self.change(key);
                self.data.insert(key, v);
            },
            Err(e) => {error!("Failed to serialize {:?}:{}",key, e);}
        }
    }
    
    pub fn set_persistent(&mut self, temp: &Attributes) -> bool {
        self.temp.remove(temp)
    }

    pub fn set_temporary(&mut self, temp: Attributes) -> bool {
        self.temp.insert(temp)
    }

    pub fn changed(&self, attribute: &Attributes) -> bool {
        self.changed.contains(attribute)
    }

    #[inline]
    fn change(&mut self, attribute: Attributes) {
        self.changed.insert(attribute);
    }

    pub fn clear_attribute(&mut self, attribute: &Attributes) {
        self.data.remove(attribute);
    }
}

pub(crate) fn update_delta<Flag: Component>(
    time: Res<Time>,
    mut states: Query<&mut AnimationState, With<Flag>>,
){
    for mut state in states.iter_mut() {
        state.set_attribute(Attributes::DELTA, time.delta_seconds());
    }
}

pub(crate) fn clear_unchanged_temp(
    mut states: Query<&mut AnimationState>,
) {
    for mut state in states.iter_mut() {
        let state = state.as_mut();
        let mut to_clear = Vec::with_capacity(state.temp.len());
        for temp in state.temp.iter() {
            if !state.changed(temp) {
                to_clear.push(*temp);
            }
        }
        for clear in to_clear.iter() {
            state.clear_attribute(clear)
        }
    }
}

pub(crate) fn clear_changed(
    mut states: Query<&mut AnimationState>
) {
    for mut state in states.iter_mut() {
        state.changed.clear();
    }
}