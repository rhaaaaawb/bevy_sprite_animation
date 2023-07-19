use crate::{
    error::BevySpriteAnimationError as Error, prelude::Attribute, AnimationFrame, AnimationMedium,
};
use bevy::prelude::*;

pub trait AnimationNode<Frame>: Send + Sync + Any {
    fn run(&self, state: &mut super::state::AnimationState) -> NodeResult<Frame>;
    fn name(&self) -> &str;
    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        context: &mut bevy_inspector_egui::Context,
    ) -> bool;
    fn id(&self) -> NodeID;
    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &AssetServer) -> Result<(), Error> {
        let _ = asset_server;
        data.push_str("serializetion for ");
        data.push_str(&self.node_type());
        data.push_str(" not implemented\n");
        Ok(())
    }
    fn node_type(&self) -> String;
    #[cfg(feature = "hash")]
    fn hash(&self) -> u64;
}

pub trait CanLoad<F> {
    fn loader() -> Box<dyn NodeLoader<F>>;
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
pub struct NodeID(u64);

use std::{any::Any, collections::HashMap};
lazy_static::lazy_static! {
    static ref NODE_ID_NAMES: std::sync::Mutex<HashMap<NodeID, String>> = {
        let map = HashMap::new();
        std::sync::Mutex::new(map)
    };
}

impl NodeID {
    pub fn as_u64(self) -> u64 {
        self.0
    }
    pub fn from_u64(id: u64) -> Self {
        NodeID(id)
    }
    pub fn from_str(data: &str) -> NodeID {
        let data = data.trim();
        let data = if data.starts_with("NodeID(") {
            if !data.ends_with(')') {
                panic!("NodeID: started with 'NodeID(' but did not end with ')'");
            }
            &data[7..data.len() - 1]
        } else {
            data
        };
        let id = if data.starts_with(|c: char| c.is_digit(10)) {
            NodeID::from_digit(data)
        } else {
            NodeID::from_name(data)
        };
        id
    }

    pub fn from_name(name: &str) -> NodeID {
        let name = name.trim();
        let id = NodeID::hash_name(name);
        let mut names = NODE_ID_NAMES.lock().unwrap();
        if !names.contains_key(&id) {
            names.insert(id, name.to_string());
        }
        id
    }

    fn hash_name(name: &str) -> NodeID {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        while hasher.finish() < 65536 {
            hasher.write_u8(0);
        }
        NodeID(hasher.finish())
    }

    fn from_digit(from: &str) -> NodeID {
        let from = from.trim();
        if from.starts_with("0x") || from.starts_with("0X") {
            NodeID::from_u64(
                u64::from_str_radix(&from[2..], 16).expect("NodeID: failed to parse hexadecimal"),
            )
        } else if from.starts_with("0b") || from.starts_with("0B") {
            NodeID::from_u64(
                u64::from_str_radix(&from[2..], 2).expect("NodeID: failed to parse binary"),
            )
        } else if from.starts_with("0o") || from.starts_with("0O") {
            NodeID::from_u64(
                u64::from_str_radix(&from[2..], 8).expect("NodeID: failed to parse octal"),
            )
        } else {
            NodeID::from_u64(
                u64::from_str_radix(from, 10).expect("NodeID: failed to parse decimal"),
            )
        }
    }

    pub fn name(&self) -> Option<String> {
        if let Some(v) = NODE_ID_NAMES.lock().unwrap().get(self) {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn name_or_id(&self) -> String {
        if let Some(name) = self.name() {
            name
        } else {
            format!("NodeID({:#018X})", self.0)
        }
    }
}

#[cfg(feature = "serialize")]
mod serde {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize)]
    struct NodeID(String);

    impl Serialize for super::NodeID {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            NodeID(format!("{:#018X}", self.0)).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for super::NodeID {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let r = NodeID::deserialize(deserializer)?;
            let res = u64::from_str_radix(&r.0[2..], 16);
            if let Ok(id) = res {
                Ok(Self(id))
            } else {
                bevy::log::error!("NodeID deserialize error {:?}", res);
                Ok(Self(0))
            }
        }
    }
}

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for NodeID {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _: Self::Attributes,
        _: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(self.to_string());
        false
    }
}

impl std::fmt::Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("NodeID({:#020X})", self.0))
    }
}

#[derive(Debug)]
pub enum NodeResult<Frame> {
    Next(NodeID),
    Done(Frame),
    Error(String),
}

impl<F> std::fmt::Display for NodeResult<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeResult::Next(id) => f.write_fmt(format_args!("Next({:#x})", id.0)),
            NodeResult::Done(_) => f.write_str("Done"),
            NodeResult::Error(_) => f.write_str("Error"),
        }
    }
}

pub trait NodeLoader<A>: 'static + Send + Sync {
    fn load(
        &mut self,
        data: &str,
        asset_server: &AssetServer,
    ) -> Result<Box<dyn AnimationNode<A>>, crate::error::BevySpriteAnimationError>;
    fn can_load(&self) -> &[&str];
}

#[derive(Component, Debug, Reflect, std::hash::Hash)]
pub struct ImageHandles {
    pub handles: Vec<Handle<Image>>,
    pub index: Attribute,
}

impl AnimationMedium for ImageHandles {
    type Frame = Handle<Image>;

    fn current_frame(&self) -> &Self::Frame {
        &self.handles[usize::from(self.index)]
    }

    fn set_frame(&mut self, index: usize) {
        self.index = index.into();
    }

    fn num_frames(&self) -> usize {
        self.handles.len()
    }

    fn current_index(&self) -> usize {
        self.index.into()
    }

    fn frame_at_index(&self, index: usize) -> &Self::Frame {
        &self.handles[index]
    }
}

impl ImageHandles {
    pub fn new(handles: Vec<Handle<Image>>, index: Attribute) -> Self {
        Self { handles, index }
    }
}

impl AnimationFrame for Handle<Image> {
    fn set_frame(&mut self, new_frame: Handle<Image>) {
        *self = new_frame;
    }
}
