use crate::error::LoadError;
use crate::prelude::*;
use crate::serde::LoadNode;
use crate::serde::ReflectLoadNode;
use bevy::asset::AssetPath;
use bevy::prelude::Handle;
use bevy::prelude::Image;
use bevy::reflect::Reflect;
use serde::Deserializer;
use crate::error::BevySpriteAnimationError as Error;

#[derive(Debug, Reflect, std::hash::Hash)]
#[reflect(LoadNode)]
pub struct IndexNode{
    name: String,
    frames: Vec<Handle<Image>>,
    is_loop: bool,
    index: Attribute,
}

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for IndexNode {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _options: Self::Attributes, _context: &mut bevy_inspector_egui::Context) -> bool {
        let mut edit = false;
        ui.collapsing("IndexNode", |ui| {
        ui.horizontal(|ui| {
            ui.label("Name: ");
            if ui.text_edit_singleline(&mut self.name).changed() {edit = true;}
        });
        if ui.checkbox(&mut self.is_loop, "loop").changed() {edit = true;};
        });
        edit
    }
}

impl IndexNode {
    #[inline(always)]
    pub fn new(name: &str, frames: &[Handle<Image>], is_loop: bool) -> IndexNode{
        IndexNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index: Attribute::IndexId(0),
        }
    }

    #[inline(always)]
    pub fn new_with_index(name: &str, frames: &[Handle<Image>], is_loop: bool, index: Attribute) -> IndexNode {
        IndexNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index,
        }
    }
}

impl AnimationNodeTrait for IndexNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn node_type(&self) -> String {
        "IndexNode".to_string()
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        assert!(self.frames.len() != 0);
        let mut index = state.index(&self.index);
        let frames = state.attribute::<usize>(&Attribute::Frames);
        index += frames;
        if index >= self.frames.len() {
            if self.is_loop {
                index %= self.frames.len();
            } else {
                index = self.frames.len() - 1;
            }
        }
        state.set_attribute(self.index.clone(), index);
        Ok(NodeResult::Done(self.frames[index].clone()))
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &bevy::prelude::AssetServer) -> Result<(), Error>
    {
        data.push_str("IndexNode(\n\t");
        data.push_str("name: \"");
        data.push_str(&self.name);
        data.push_str("\",\n\tframes: [\n\t");
        for frame in self.frames.iter() {
            if let Some(path) = asset_server.get_handle_path(frame) {
                data.push_str(path.path().to_str().unwrap())
            } else {
                return Err(Error::AssetPathNotFound(frame.clone_weak()));
            }
            data.push_str(",\n\t");
        }
        data.push_str("],\n\t");
        data.push_str(&format!("is_loop: {},\n\t",self.is_loop));
        data.push_str("index: ");
        data.push_str(&ron::to_string(&self.index)?);
        data.push_str(",\n\t),\n");
        Ok(())
    }

    fn id(&self) -> NodeId {
        NodeId::from_name(&self.name)
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl LoadNode for IndexNode {
    fn load<'b>(s: &str, load_context: &mut bevy::asset::LoadContext<'b>, dependencies: &mut Vec<AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("IndexNode", &[], IndexLoader(load_context, dependencies)) {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError{code: e, position: ron::de::Position{line: 0, col: 0}})),
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

struct IndexLoader<'de, 'b: 'de>(&'de mut bevy::asset::LoadContext<'b>, &'de mut Vec<AssetPath<'static>>);

impl<'de, 'b: 'de> serde::de::Visitor<'de> for IndexLoader<'de, 'b> {
    type Value = IndexNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        use serde::de::Error;
        let mut name = None;
        let mut frames = None;
        let mut is_loop = false;
        let mut index = Attribute::IndexId(0);
        while let Some(key) = map.next_key::<Fileds>()? {
            match key {
                Fileds::Name => name = Some(map.next_value::<String>()?),
                Fileds::Frames => frames = Some(map.next_value::<Vec<String>>()?),
                Fileds::IsLoop => is_loop = map.next_value::<bool>()?,
                Fileds::Index => index = map.next_value::<Attribute>()?,
            }
        }
        let Some(frames) = frames else {return Err(Error::missing_field("Frames"));};
        let Some(name) = name else {return Err(Error::missing_field("Name"));};
        let mut handles = Vec::with_capacity(frames.len());
        for frame in frames {
            handles.push(self.0.get_handle::<_, Image>(&frame));
            self.1.push(frame.into());
        }
        Ok(IndexNode {
            frames: handles,
            name,
            is_loop,
            index
        })
    }
}