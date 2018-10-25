use crate::dsl::compiler::compile;
use crate::dsl::compiler::ObjectType;
use crate::dsl::compiler::PropertyList;
use crate::dsl::validation;
use serde_derive::Serialize;
use std::collections::HashMap;
use crate::dsl::validation::ValidatedSchema;

pub struct Generator {
    compiled_schema: ValidatedSchema,
}

impl Generator {
    pub fn with(yaml: serde_yaml::Value) -> Result<Self, validation::Error> {
        Ok(Generator::new(compile(yaml)?))
    }

    fn new(compiled_schema: ValidatedSchema) -> Self {
        Generator { compiled_schema }
    }

    pub fn generate(self) -> (serde_json::Value, serde_json::Value) {
        let property_names = match self.compiled_schema.properties {
            Some(ref list) => Some(list.clone().property_names),
            None => None,
        };

        let schema = JsonSchema {
            version: self.compiled_schema.version,
            title: self.compiled_schema.title.to_string(),
            properties: self.compiled_schema.properties,
            required: property_names.clone(),
            order: property_names.clone(),
            type_spec: crate::dsl::compiler::ObjectType::Object,
            schema_url: "http://json-schema.org/draft-04/schema#".to_string(),
        };

        let ui_object = UiObject(HashMap::new());

        (
            serde_json::to_value(schema).expect("Internal error: inconsistent schema: json schema"),
            serde_json::to_value(ui_object).expect("Internal error: inconsistent schema: ui object"),
        )
    }
}

#[derive(Serialize)]
struct JsonSchema {
    #[serde(rename = "$$version")]
    version: u64,
    #[serde(rename = "$schema")]
    schema_url: String,
    #[serde(rename = "type")]
    type_spec: ObjectType,
    title: String,
    #[serde(
        rename = "properties",
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::ui_configuration::to_json_schema::serialize_property_list"
    )]
    properties: Option<PropertyList>,
    #[serde(rename = "$$order", skip_serializing_if = "Option::is_none")]
    order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
}

#[derive(Serialize)]
struct UiObject(HashMap<String, UiObjectProperty>);

#[derive(Serialize)]
struct UiObjectProperty {
    #[serde(rename = "ui:help")]
    help: String,
    #[serde(rename = "ui:warning")]
    warning: String,
    #[serde(rename = "ui:description")]
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::validation::validate;
    use crate::dsl::compiler::SourceSchema;

    #[test]
    fn hardcode_a_type() -> Result <(), validation::Error> {
        let generator = Generator::new(validate(SourceSchema::empty())?);

        let (json_schema, _) = generator.generate();

        assert_eq!(json_schema["type"], "object");
        Ok(())
    }

    #[test]
    fn hardcode_a_schema_url() -> Result <(), validation::Error> {
        let generator = Generator::new(validate(SourceSchema::empty())?);

        let (json_schema, _) = generator.generate();

        assert_eq!(json_schema["$schema"], "http://json-schema.org/draft-04/schema#");
        Ok(())
    }

    #[test]
    fn pass_title_through() -> Result <(), validation::Error> {
        let schema = validate(SourceSchema::with("some title", 1))?;
        let generator = Generator::new(schema);

        let (json_schema, _) = generator.generate();

        assert_eq!(json_schema["title"], "some title");
        Ok(())
    }

    #[test]
    fn generate_ui_object() -> Result <(), validation::Error> {
        let generator = Generator::new(validate(SourceSchema::empty())?);

        let (_, ui_object) = generator.generate();

        assert!(ui_object.is_object());
        Ok(())
    }

    #[test]
    fn generate_json_schema() -> Result <(), validation::Error> {
        let generator = Generator::new(validate(SourceSchema::empty())?);

        let (json_schema, _) = generator.generate();

        assert!(json_schema.is_object());
        Ok(())
    }

    impl SourceSchema {
        fn empty() -> Self {
            SourceSchema {
                title: String::new(),
                version: 1,
                properties: None,
            }
        }

        fn with(title: &str, version: u64) -> Self {
            SourceSchema {
                title: title.to_string(),
                version,
                properties: None,
            }
        }
    }
}
