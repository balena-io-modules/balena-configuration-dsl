use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use regex::Regex;
use serde_derive::Deserialize;
use serde_json::{Number, Value};
use yaml_merge_keys::merge_keys_serde;

// Reexport everything except mapping, which is a public module
pub use self::{
    property::Property,
    r#enum::EnumEntry,
    r#type::{PrimitiveType, Type},
    unique_items::UniqueItems,
    version::Version,
};

use crate::{
    deref::OptionDeref,
    error::{Error, ResultExt},
};

mod r#enum;
pub mod mapping;
mod property;
mod r#type;
mod unique_items;
mod version;

/// JellySchema structure
///
/// # WARNING
///
/// JellySchema is YAML based. Althought, we're deserializing everything into
/// `serde_json` structures like `Value`, `Number` or Rust types. The reason is
/// that we're generating JSON values from the JellySchema. And this allows us
/// to catch missing JSON features (when compared with YAML) during deserialization.
#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(default)]
    version: Option<Version>,
    //
    // Mapping extension
    //
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mapping: Option<mapping::Mapping>,
    //
    // Any instance type validation keywords
    //
    #[serde(default, rename = "type", deserialize_with = "deserialize_from_str")]
    r#type: Type,
    #[serde(default, rename = "const", skip_serializing_if = "Option::is_none")]
    r#const: Option<Value>,
    #[serde(default, rename = "default", skip_serializing_if = "Option::is_none")]
    r#default: Option<Value>,
    #[serde(default, rename = "enum", skip_serializing_if = "Vec::is_empty")]
    r#enum: Vec<EnumEntry>,
    #[serde(
        default,
        deserialize_with = "deserialize_as_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    formula: Option<String>,
    #[serde(default, rename = "readOnly")]
    read_only: bool,
    #[serde(default, rename = "writeOnly")]
    write_only: bool,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    hidden: bool,
    //
    // Object validation keywords
    //
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    properties: Vec<Property>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    keys: Option<Box<Schema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    values: Option<Box<Schema>>,
    #[serde(default, rename = "additionalProperties")]
    additional_properties: bool,
    //
    // StringList keywords
    //
    #[serde(default, skip_serializing_if = "Option::is_none")]
    separator: Option<String>,
    //
    // Annotation keywords
    //
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    //
    // Array validation keywords
    //
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_struct_or_vec"
    )]
    items: Vec<Schema>,
    #[serde(default, rename = "maxItems", skip_serializing_if = "Option::is_none")]
    max_items: Option<usize>,
    #[serde(default, rename = "minItems", skip_serializing_if = "Option::is_none")]
    min_items: Option<usize>,
    #[serde(default, rename = "uniqueItems")]
    unique_items: UniqueItems,
    #[serde(default)]
    orderable: Option<bool>,
    #[serde(default)]
    addable: Option<bool>,
    #[serde(default)]
    removable: Option<bool>,
    //
    // Number validation keywords
    //
    #[serde(default, rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    multiple_of: Option<Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max: Option<Number>,
    #[serde(default, rename = "exclusiveMax", skip_serializing_if = "Option::is_none")]
    exclusive_max: Option<Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min: Option<Number>,
    #[serde(default, rename = "exclusiveMin", skip_serializing_if = "Option::is_none")]
    exclusive_min: Option<Number>,
    //
    // String based types validation keywords
    //
    #[serde(default, rename = "maxLength", skip_serializing_if = "Option::is_none")]
    max_length: Option<usize>,
    #[serde(default, rename = "minLength", skip_serializing_if = "Option::is_none")]
    min_length: Option<usize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_option_from_str"
    )]
    pattern: Option<Regex>,
}

impl Schema {
    pub fn version(&self) -> Option<u8> {
        self.version.as_ref().map(Version::value)
    }
}

//
// Any instance type
//
impl Schema {
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn r#const(&self) -> Option<&Value> {
        self.r#const.as_ref()
    }

    pub fn r#default(&self) -> Option<&Value> {
        self.r#default.as_ref()
    }

    pub fn r#enum(&self) -> &[EnumEntry] {
        self.r#enum.as_slice()
    }

    pub fn formula(&self) -> Option<&str> {
        self.formula.as_deref()
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn write_only(&self) -> bool {
        self.write_only
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }
}

//
// Mapping extension
//
impl Schema {
    pub fn mapping(&self) -> Option<&mapping::Mapping> {
        self.mapping.as_ref()
    }
}

//
// Object validation keywords
//
impl Schema {
    pub fn properties(&self) -> &[Property] {
        self.properties.as_slice()
    }

    pub fn keys(&self) -> Option<&Schema> {
        self.keys.as_deref()
    }

    pub fn values(&self) -> Option<&Schema> {
        self.values.as_deref()
    }

    pub fn additional_properties(&self) -> bool {
        self.additional_properties
    }
}

//
// StringList keywords
//
impl Schema {
    pub fn separator(&self) -> Option<&str> {
        self.separator.as_deref()
    }
}

//
// Annotation keywords
//
impl Schema {
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

//
// Number validation keywords
//
impl Schema {
    pub fn multiple_of(&self) -> Option<&Number> {
        self.multiple_of.as_ref()
    }

    pub fn max(&self) -> Option<&Number> {
        self.max.as_ref()
    }

    pub fn min(&self) -> Option<&Number> {
        self.min.as_ref()
    }

    pub fn exclusive_max(&self) -> Option<&Number> {
        self.exclusive_max.as_ref()
    }

    pub fn exclusive_min(&self) -> Option<&Number> {
        self.exclusive_min.as_ref()
    }
}

//
// String based types validation keywords
//
impl Schema {
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    pub fn min_length(&self) -> Option<usize> {
        self.min_length
    }

    pub fn pattern(&self) -> Option<&Regex> {
        self.pattern.as_ref()
    }
}

//
// Array validation keywords
//
impl Schema {
    pub fn items(&self) -> &[Schema] {
        self.items.as_slice()
    }

    pub fn max_items(&self) -> Option<usize> {
        self.max_items
    }

    pub fn min_items(&self) -> Option<usize> {
        self.min_items
    }

    pub fn unique_items(&self) -> &UniqueItems {
        &self.unique_items
    }

    pub fn addable(&self) -> Option<bool> {
        self.addable
    }

    pub fn removable(&self) -> Option<bool> {
        self.removable
    }

    pub fn orderable(&self) -> Option<bool> {
        self.orderable
    }
}

impl FromStr for Schema {
    type Err = Error;

    fn from_str(s: &str) -> Result<Schema, Error> {
        let value: serde_yaml::Value = serde_yaml::from_str(s)
            .map_err(|_| Error::with_message("unable to parse yaml"))
            .context("input", s.to_string())?;

        let merged_value = merge_keys_serde(value).map_err(|_| Error::with_message("unable to merge yaml keys"))?;

        let schema: Schema = serde_yaml::from_value(merged_value)
            .map_err(|_| Error::with_message("unable to create schema from yaml with merged keys"))?;
        Ok(schema)
    }
}

fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: std::fmt::Display,
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_as_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = match serde::de::Deserialize::deserialize(deserializer)? {
        Value::Bool(x) => format!("{}", x),
        Value::Number(x) => format!("{}", x),
        Value::String(x) => x.clone(),
        _ => return Err(serde::de::Error::custom("unable to deserialize as string")),
    };

    Ok(Some(s))
}

fn deserialize_option_from_str<'de, S, D>(deserializer: D) -> Result<Option<S>, D::Error>
where
    S: FromStr,
    S::Err: std::fmt::Display,
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(Some(S::from_str(&s).map_err(serde::de::Error::custom)?))
}

fn deserialize_struct_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: serde::de::Deserialize<'de>,
    D: serde::de::Deserializer<'de>,
{
    struct StructOrVec<T>(PhantomData<T>);

    impl<'de, T> serde::de::Visitor<'de> for StructOrVec<T>
    where
        T: serde::de::Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("schema or list of schemas")
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            serde::de::Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(visitor)).map(|x| vec![x])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            serde::de::Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StructOrVec(PhantomData))
}
