use crate::{
    macros::newtype_uuid,
    settings::{patch::patch_json, Overrides},
};
use anyhow::Result;

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::action::PipelineAction;

newtype_uuid!(PipelineActionId);
newtype_uuid!(PipelineActionDefinitionId);
newtype_uuid!(PipelineDefinitionId);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub tags: Vec<String>,
    pub id: PipelineDefinitionId,
    pub description: String,
    pub action: PipelineActionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The id of the action
    pub id: PipelineActionDefinitionId,
    /// The name of the action
    pub name: String,
    /// An optional description of what the action does.
    pub description: Option<String>,
    /// The value of the pipeline action
    pub selection: Selection,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
}

impl PipelineActionDefinition {
    pub fn new(
        name: String,
        description: Option<String>,
        id: PipelineActionDefinitionId,
        optional: Option<bool>,
        selection: Selection,
    ) -> Self {
        Self {
            name,
            description,
            id,
            optional,
            selection,
        }
    }
}

// fn value_schema(gen: &mut SchemaGenerator) -> Schema {
//     let mut schema: SchemaObject = String::json_schema(gen).into();
//     schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
//         InstanceType::Object,
//     )));
//     schema.object = None;
//     schema.string = None;
//     schema.into()
// }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub enum Selection {
    Action(PipelineAction),
    OneOf {
        selection: PipelineActionDefinitionId,
        actions: IndexMap<PipelineActionDefinitionId, PipelineActionDefinition>,
    },
    AllOf(Vec<PipelineActionDefinition>),
}

impl PipelineDefinition {
    pub fn new(
        id: PipelineDefinitionId,
        name: String,
        description: String,
        tags: Vec<String>,
        selection: Selection,
    ) -> Self {
        let action = PipelineActionDefinition {
            id: PipelineActionDefinitionId::from_uuid(id.0),
            name: "root".to_string(),
            description: None,
            selection,
            optional: None,
        };
        Self {
            action,
            name,
            tags,
            id,
            description,
        }
    }

    pub fn patched_with(&self, overrides: Overrides) -> Self {
        let mut patched = (*self).clone();
        for (id, value) in overrides.enabled.into_iter() {
            patched.patch_enabled(&id, value);
        }

        for (id, value) in overrides.fields.into_iter() {
            patched.patch_override(&id, value);
        }
        patched
    }

    fn get_definition_mut(
        &mut self,
        id: &PipelineActionDefinitionId,
    ) -> Option<&mut PipelineActionDefinition> {
        fn get_definition_rec<'a>(
            def: &'a mut PipelineActionDefinition,
            id: &PipelineActionDefinitionId,
        ) -> Option<&'a mut PipelineActionDefinition> {
            if def.id == *id {
                return Some(def);
            }

            match def.selection {
                Selection::Action(_) => None,
                Selection::OneOf {
                    ref mut actions, ..
                } => actions
                    .iter_mut()
                    .fold(None, |acc, a| if a.0 == id { Some(a.1) } else { acc }),
                Selection::AllOf(ref mut definitions) => definitions
                    .iter_mut()
                    .fold(None, |acc, d| if d.id == *id { Some(d) } else { acc }),
            }
        }

        get_definition_rec(&mut self.action, id)
    }

    fn patch_enabled(&mut self, id: &PipelineActionDefinitionId, value: bool) {
        let def = self.get_definition_mut(id);
        if let Some(def) = def {
            def.optional = def.optional.map(|_| value);
        }
    }

    fn patch_override(&mut self, id: &PipelineActionDefinitionId, value: Value) {
        let def = self.get_definition_mut(id);
        if let Some(def) = def {
            def.selection = match &def.selection {
                Selection::Action(action) => {
                    let current_json = serde_json::to_value(action).unwrap();
                    Selection::Action(
                        serde_json::from_value(patch_json(current_json, value)).unwrap(),
                    )
                }
                Selection::OneOf { actions, .. } => {
                    let new_selection = value["selection"].as_str().unwrap();
                    Selection::OneOf {
                        selection: PipelineActionDefinitionId::parse(new_selection),
                        actions: actions.clone(), // TODO::avoid this clone
                    }
                }
                Selection::AllOf(_) => panic!("AllOf definitions are not patchable!"),
            }
        }
    }
}
