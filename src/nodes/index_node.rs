use crate::error::LoadError;
use crate::node_core::IntoNodeFrames;
use crate::node_core::IntoNodeFramesTrait;
use crate::node_core::NodeFrames;
use crate::node_core::NodeFramesTrait;
use crate::prelude::*;
use crate::serde::LoadNode;
use crate::serde::ReflectLoadNode;
use bevy::asset::AssetPath;
use bevy::prelude::Handle;
use bevy::prelude::Image;
use bevy::reflect::Reflect;
use serde::Deserializer;

#[derive(Debug, Reflect)]
#[reflect(LoadNode)]
pub struct IndexNode {
    id: Option<NodeId<'static>>,
    name: String,
    frames: NodeFrames<()>,
    is_loop: bool,
    index: Attribute,
}

#[cfg(feature = "sprite_sheet")]
impl IndexNode {
    #[inline(always)]
    pub fn new(
        name: &str,
        frames: Handle<TextureAtlas>,
        min_i: usize,
        max_i: usize,
        is_loop: bool,
    ) -> IndexNode {
        IndexNode {
            id: None,
            name: name.to_string(),
            frames: NodeFrames::new(frames, min_i, max_i, Vec::new()),
            is_loop,
            index: Attribute::IndexId(0),
        }
    }

    #[inline(always)]
    pub fn new_with_index(
        name: &str,
        frames: Handle<TextureAtlas>,
        min_i: usize,
        max_i: usize,
        is_loop: bool,
        index: Attribute,
    ) -> IndexNode {
        IndexNode {
            id: None,
            name: name.to_string(),
            frames: NodeFrames::new(frames, min_i, max_i, Vec::new()),
            is_loop,
            index,
        }
    }
}

#[cfg(all(feature = "image_vec", not(feature = "sprite_sheet")))]
impl IndexNode {
    #[inline(always)]
    pub fn new(name: &str, frames: &[Handle<Image>], is_loop: bool) -> IndexNode {
        IndexNode {
            id: None,
            name: name.to_string(),
            frames: frames.into(),
            is_loop,
            index: Attribute::IndexId(0),
        }
    }

    #[inline(always)]
    pub fn new_with_index(
        name: &str,
        frames: &[Handle<Image>],
        is_loop: bool,
        index: Attribute,
    ) -> IndexNode {
        IndexNode {
            id: None,
            name: name.to_string(),
            frames: frames.into(),
            is_loop,
            index,
        }
    }
}

impl AnimationNodeTrait for IndexNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        assert!(self.frames.num_frames() != 0);
        let mut index = state.index(&self.index);
        let frames = state.attribute::<usize>(&Attribute::Frames);
        index += frames;
        if index >= self.frames.num_frames() {
            if self.is_loop {
                index %= self.frames.num_frames();
            } else {
                index = self.frames.num_frames() - 1;
            }
        }
        state.set_attribute(self.index.clone(), index);
        Ok(NodeResult::Done((
            index,
            self.frames.frame_handle(index).clone(),
        )))
    }

    fn id(&self) -> NodeId {
        if let Some(id) = &self.id {
            id.to_static()
        } else {
            NodeId::from_name(&self.name)
        }
    }

    fn set_id(&mut self, id: NodeId<'_>) {
        self.id = Some(id.to_static());
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        self.frames.dot(this, out, asset_server);
    }
}

impl LoadNode for IndexNode {
    fn load<'b>(
        s: &str,
        load_context: &mut bevy::asset::LoadContext<'b>,
        dependencies: &mut Vec<AssetPath<'static>>,
    ) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("IndexNode", &[], IndexLoader(load_context, dependencies)) {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError {
                code: e,
                position: ron::de::Position { line: 0, col: 0 },
            })),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Fileds {
    Name,
    Frames,
    IsLoop,
    Index,
}

struct IndexLoader<'de, 'b: 'de>(
    &'de mut bevy::asset::LoadContext<'b>,
    &'de mut Vec<AssetPath<'static>>,
);

impl<'de, 'b: 'de> serde::de::Visitor<'de> for IndexLoader<'de, 'b> {
    type Value = IndexNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        use serde::de::Error;
        let mut name = None;
        let mut frames = None;
        let mut is_loop = false;
        let mut index = Attribute::IndexId(0);
        while let Some(key) = map.next_key::<Fileds>()? {
            match key {
                Fileds::Name => name = Some(map.next_value::<String>()?),
                Fileds::Frames => frames = Some(map.next_value::<IntoNodeFrames<()>>()?),
                Fileds::IsLoop => is_loop = map.next_value::<bool>()?,
                Fileds::Index => index = map.next_value::<Attribute>()?,
            }
        }
        let Some(frames) = frames else {return Err(Error::missing_field("Frames"));};
        let Some(name) = name else {return Err(Error::missing_field("Name"));};

        Ok(IndexNode {
            id: None,
            frames: frames.to_node_frames(self.0, self.1),
            name,
            is_loop,
            index,
        })
    }
}
