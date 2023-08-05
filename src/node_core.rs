use std::fmt::Debug;

use bevy::{
    asset::{AssetPath, LoadContext},
    prelude::*,
};

use crate::{
    error::{BevySpriteAnimationError as Error, RunError},
    prelude::*,
};

pub trait AnimationNodeTrait: Reflect {
    fn run(&self, state: &mut super::state::AnimationState) -> Result<NodeResult, RunError>;
    fn name(&self) -> &str {
        self.reflect_short_type_path()
    }
    fn id(&self) -> NodeId<'_>;
    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &AssetServer) -> Result<(), Error> {
        let _ = asset_server;
        data.push_str(self.reflect_type_path());
        data.push_str("(serializetion not implemented)\n");
        Ok(())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AnimationNode(")?;
        f.write_str(self.type_name())?;
        f.write_str(")")
    }

    #[cfg(feature = "dot")]
    #[allow(unused)]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name()));
    }

    fn set_id(&mut self, id: NodeId<'_>);
}

#[derive(Debug)]
pub enum NodeResult {
    Next(NodeId<'static>),
    Done((usize, FrameHandle)),
}

impl std::fmt::Display for NodeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeResult::Next(id) => f.write_fmt(format_args!("Next({:#?})", id)),
            NodeResult::Done(_) => f.write_str("Done"),
        }
    }
}

pub use node_frames::*;

#[cfg(feature = "sprite_sheet")]
mod node_frames {
    use bevy::{
        asset::LoadContext,
        prelude::Handle,
        reflect::Reflect,
        sprite::{TextureAtlas, TextureAtlasSprite},
    };

    use super::{FrameSpriteTrait, NodeFramesTrait};

    pub type FrameHandle = Handle<TextureAtlas>;
    pub type FrameSprite = TextureAtlasSprite;

    #[derive(Debug, Reflect)]
    pub struct NodeFrames<T> {
        min_i: usize,
        max_i: usize,
        atlas: Handle<TextureAtlas>,
        extra_data: Vec<T>,
    }

    impl<T> NodeFrames<T> {
        pub fn new(
            atlas: Handle<TextureAtlas>,
            min_i: usize,
            max_i: usize,
            extra_data: Vec<T>,
        ) -> Self {
            Self {
                min_i,
                max_i,
                atlas,
                extra_data,
            }
        }
    }

    impl<T> NodeFramesTrait<T> for NodeFrames<T> {
        fn num_frames(&self) -> usize {
            self.max_i - self.min_i
        }

        fn get_extra(&self, index: usize) -> &T {
            self.extra_data.get(index - self.min_i).unwrap()
        }

        fn frame_handle(&self, _index: usize) -> &FrameHandle {
            &self.atlas
        }

        #[cfg(feature = "dot")]
        fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer) {
            todo!()
        }
    }

    impl FrameSpriteTrait for TextureAtlasSprite {
        fn set_frame_index(&mut self, index: usize) {
            self.index = index;
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct IntoNodeFrames<T> {
        min_i: usize,
        max_i: usize,
        atlas: String,
        extra_data: Vec<T>,
    }

    impl<T> IntoNodeFramesTrait<T> for IntoNodeFrames<T> {
        fn to_node_frames<'b>(
            self,
            ctx: &LoadContext<'b>,
            dependencies: &mut Vec<AssetPath<'static>>,
        ) -> NodeFrames<T> {
            let atlas = ctx.get_handle::<_, TextureAtlas>(&self.atlas);
            dependencies.push(self.atlas.into());
            NodeFrames {
                min_i: self.min_i,
                max_i: self.max_i,
                atlas,
                extra_data: self.extra_data,
            }
        }
    }
}

#[cfg(all(feature = "image_vec", not(feature = "sprite_sheet")))]
mod node_frames {
    use bevy::asset::{AssetPath, LoadContext};

    use super::*;

    pub type FrameHandle = Handle<Image>;
    pub type FrameSprite = Sprite;

    #[derive(Debug, Reflect)]
    pub struct NodeFrames<T> {
        frames: Vec<(Handle<Image>, T)>,
    }

    impl<T> NodeFramesTrait<T> for NodeFrames<T> {
        fn num_frames(&self) -> usize {
            self.frames.len()
        }

        fn get_extra(&self, index: usize) -> &T {
            &self.frames.get(index).unwrap().1
        }

        fn frame_handle(&self, index: usize) -> &FrameHandle {
            &self.frames.get(index).unwrap().0
        }

        #[cfg(feature = "dot")]
        fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer) {
            todo!()
        }
    }

    impl<T: Clone> From<&[(Handle<Image>, T)]> for NodeFrames<T> {
        fn from(value: &[(Handle<Image>, T)]) -> Self {
            Self {
                frames: value.to_vec(),
            }
        }
    }

    impl From<&[Handle<Image>]> for NodeFrames<()> {
        fn from(value: &[Handle<Image>]) -> Self {
            Self {
                frames: value.iter().map(|h| (h.clone(), ())).collect(),
            }
        }
    }

    impl<T: Clone> From<Vec<(Handle<Image>, T)>> for NodeFrames<T> {
        fn from(value: Vec<(Handle<Image>, T)>) -> Self {
            Self { frames: value }
        }
    }

    impl From<Vec<Handle<Image>>> for NodeFrames<()> {
        fn from(value: Vec<Handle<Image>>) -> Self {
            Self {
                frames: value.into_iter().map(|h| (h.clone(), ())).collect(),
            }
        }
    }

    impl FrameSpriteTrait for Sprite {
        fn set_frame_index(&mut self, _index: usize) {
            // nop
        }
    }

    pub type IntoNodeFrames<T> = Vec<(String, T)>;

    impl<T> IntoNodeFramesTrait<T> for IntoNodeFrames<T> {
        fn to_node_frames<'b>(
            self,
            ctx: &LoadContext<'b>,
            dependencies: &mut Vec<AssetPath<'static>>,
        ) -> NodeFrames<T> {
            let frames = self
                .into_iter()
                .map(|(path, extra)| {
                    let handle = ctx.get_handle::<_, Image>(&path);
                    dependencies.push(path.into());
                    (handle, extra)
                })
                .collect();

            NodeFrames { frames }
        }
    }
}

pub trait NodeFramesTrait<T> {
    fn num_frames(&self) -> usize;

    fn get_extra(&self, index: usize) -> &T;

    fn frame_handle(&self, index: usize) -> &FrameHandle;

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer);
}

pub trait FrameSpriteTrait: Component {
    fn set_frame_index(&mut self, index: usize);
}

pub trait IntoNodeFramesTrait<T> {
    fn to_node_frames<'b>(
        self,
        ctx: &LoadContext<'b>,
        dependencies: &mut Vec<AssetPath<'static>>,
    ) -> NodeFrames<T>;
}
