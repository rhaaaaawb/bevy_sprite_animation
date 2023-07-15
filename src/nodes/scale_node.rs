use crate::serde::ReflectLoadNode;
use crate::{node_core::CanLoad, serde::LoadNode};
use crate::prelude::*;
use bevy::reflect::Reflect;
use serde::{Serialize, Deserialize, Deserializer};
use crate::error::{BevySpriteAnimationError as Error, LoadError};

#[derive(Debug, Serialize, Deserialize, Reflect)]
#[reflect(LoadNode)]
pub struct ScaleNode{
    name: String,
    scale: Attribute,
    next: NodeId<'static>,
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

impl ScaleNode {
    #[inline(always)]
    pub fn new(name: &str, scale: Attribute, next: NodeId<'static>) -> ScaleNode {
        ScaleNode { 
            name: name.to_string(),
            scale,
            next
        }
    }

    #[inline(always)]
    pub fn new_with_index(name: &str, scale: Attribute, next: NodeId<'static>) -> ScaleNode {
        ScaleNode { 
            name: name.to_string(),
            scale,
            next,
        }
    }
}

#[cfg(feature = "serialize")]
impl CanLoad for ScaleNode {
    fn loader() -> Box<dyn NodeLoader> {
        Box::new(ScaleNodeLoader)
    }
}

impl AnimationNodeTrait for ScaleNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn node_type(&self) -> String {
        "ScaleNode".to_string()
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        let rem_time = state.attribute::<f32>(&Attribute::TimeThisFrame);
        let frames = *state.attribute::<usize>(&Attribute::Frames);
        let last = state.attribute::<f32>(&Attribute::LastFPS);
        let scale = state.get_attribute::<f32>(&self.scale).cloned().unwrap_or(1.);
        let mut frame_time = last * frames as f32 + rem_time;
        let width = last * scale;
        let frames = (frame_time / width).floor();
        frame_time -= frames * width;

        state.set_attribute(Attribute::LastFPS, last * scale);
        state.set_attribute(Attribute::TimeThisFrame, frame_time);
        state.set_attribute(Attribute::Frames, frames as usize);
        Ok(NodeResult::Next(self.next.to_static()))
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, _: &bevy::prelude::AssetServer) -> Result<(), Error>
    {
        let mut buf =  Vec::new();
        let pretty = ron::ser::PrettyConfig::default().new_line("\n\t".to_string());
        let mut serializer = ron::Serializer::new(&mut buf, Some(pretty))?;
        serde::Serialize::serialize(self, &mut serializer)?;
        data.push_str(&String::from_utf8_lossy(&buf));
        data.push(',');
        data.push('\n');
        Ok(())
    }

    fn id(&self) -> NodeId {
        NodeId::from_name(&self.name)
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        this.dot(out);
        out.push_str(" -> ");
        self.next.dot(out);
        out.push_str(&format!("[label=\"{}\"];\n", self.scale));
    }
}

#[cfg(feature = "serialize")]
pub use loader::ScaleNodeLoader;

#[cfg(feature = "serialize")]
mod loader {
use crate::node_core::NodeLoader;
use super::ScaleNode;

use crate::prelude::{AnimationNodeTrait, BevySpriteAnimationError as Error};
pub struct  ScaleNodeLoader;

impl NodeLoader for ScaleNodeLoader {
    fn load(&self, data: &str, _: &bevy::prelude::AssetServer) -> Result<Box<dyn AnimationNodeTrait>, Error> {
        Ok(Box::new(ron::from_str::<ScaleNode>(data)?))
    }

    fn can_load(&self) -> &[&str] {
        &["ScaleNode"]
    }
}
}

impl LoadNode for ScaleNode {
    fn load<'b>(s: &str, _: &mut bevy::asset::LoadContext<'b>, _: &mut Vec<bevy::asset::AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("ScaleNode", &[], ScaleLoader) {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError{code: e, position: ron::de::Position{line: 0, col: 0}})),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Fileds {
    Name,
    Scale,
    Next,
}

struct ScaleLoader;

impl<'de> serde::de::Visitor<'de> for ScaleLoader {
    type Value = ScaleNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        use serde::de::Error;
            let mut name = None;
            let mut scale = None;
            let mut next = None;
        while let Some(key) = map.next_key::<Fileds>()? {
            match key {
                Fileds::Name => name = Some(map.next_value::<String>()?),
                Fileds::Scale => scale = Some(map.next_value::<Attribute>()?),
                Fileds::Next => next = Some(map.next_value::<NodeId>()?),
            }
        }
        let Some(scale) = scale else {return Err(Error::missing_field("Scale"));};
        let Some(name) = name else {return Err(Error::missing_field("Name"));};
        let Some(next) = next else {return Err(Error::missing_field("Next"));};
        Ok(ScaleNode {
            name,
            scale,
            next
        })
    }
}