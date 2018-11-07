use crate::dsl::object_types::bounds::BooleanObjectBounds;
use crate::dsl::object_types::bounds::EnumerationValue;
use crate::dsl::object_types::bounds::IntegerBound;
use crate::dsl::object_types::bounds::IntegerObjectBounds;
use crate::dsl::object_types::bounds::StringLength;
use crate::dsl::object_types::bounds::StringObjectBounds;
use crate::dsl::object_types::deserialization::deserialize_integer;
use crate::dsl::schema::DisplayInformation;
use heck::MixedCase;
use regex::Regex;
use serde::de::Error;
use serde_yaml::Mapping;
use serde_yaml::Value;

// fixme this function is not the best function ;)
pub fn deserialize_string_object_bounds<E>(mapping: &Mapping) -> Result<Option<StringObjectBounds>, E>
where
    E: Error,
{
    let enumeration_values = deserialize_enumeration_values(&mapping)?;
    let constant_value = deserialize_constant_value(&mapping)?.map(|value| vec![value]);
    let pattern = deserialize_pattern(&mapping)?;
    let length = deserialize_length_bounds(&mapping)?;

    if (enumeration_values.is_some()) && constant_value.is_some() {
        return Err(Error::custom("cannot have both enum and const defined"));
    }

    let possible_values = enumeration_values.or(constant_value);

    if possible_values.is_some() && pattern.is_some() {
        return Err(Error::custom("cannot have both pattern set and enum/const bound"));
    }

    if possible_values.is_some() && length.is_some() {
        return Err(Error::custom("cannot have both length set and enum/const bound"));
    }

    let result = {
        if let Some(values) = possible_values {
            Some(StringObjectBounds::PossibleValues(values))
        } else if let Some(pattern) = pattern {
            Some(StringObjectBounds::Pattern(pattern))
        } else if let Some(length) = length {
            Some(length)
        } else {
            None
        }
    };

    Ok(result)
}

pub fn deserialize_length_bounds<E>(mapping: &Mapping) -> Result<Option<StringObjectBounds>, E>
where
    E: Error,
{
    let max_length = deserialize_integer("maxLength", &mapping)?;
    let min_length = deserialize_integer("minLength", &mapping)?;
    if max_length.is_some() || min_length.is_some() {
        Ok(Some(StringObjectBounds::Length(StringLength {
            minimum: min_length,
            maximum: max_length,
        })))
    } else {
        Ok(None)
    }
}

pub fn deserialize_integer_bounds<E>(mapping: &Mapping) -> Result<Option<IntegerObjectBounds>, E>
where
    E: Error,
{
    let maximum = deserialize_integer_bound("maximum", mapping)?;
    let minimum = deserialize_integer_bound("minimum", mapping)?;
    let multiple_of = deserialize_integer("multipleOf", mapping)?;

    if maximum.is_some() || minimum.is_some() || multiple_of.is_some() {
        Ok(Some(IntegerObjectBounds {
            minimum,
            maximum,
            multiple_of,
        }))
    } else {
        Ok(None)
    }
}

pub fn deserialize_boolean_object_bounds<E>(mapping: &Mapping) -> Result<Option<BooleanObjectBounds>, E>
where
    E: Error,
{
    let default_key = Value::from("default");

    match mapping.get(&default_key) {
        Some(default) => match default {
            Value::Bool(value) => Ok(Some(BooleanObjectBounds::DefaultValue(*value))),
            _ => Err(Error::custom(format!(
                "cannot deserialize default value - {:#?} is not a boolean",
                default
            ))),
        },
        None => Ok(None),
    }
}

fn deserialize_enumeration_values<E>(mapping: &Mapping) -> Result<Option<Vec<EnumerationValue>>, E>
where
    E: Error,
{
    let enum_key = Value::from("enum");
    match mapping.get(&enum_key) {
        Some(mapping) => match mapping {
            Value::Sequence(sequence) => {
                let result = sequence
                    .iter()
                    .map(|definition| enumeration_definition_to_enumeration_value(definition))
                    .collect::<Result<Vec<EnumerationValue>, E>>()?;
                if !result.is_empty() {
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            _ => Err(Error::custom(format!("enum `{:#?}` is not a sequence", mapping))),
        },
        None => Ok(None),
    }
}

fn deserialize_pattern<E>(mapping: &Mapping) -> Result<Option<Regex>, E>
where
    E: Error,
{
    let pattern_key = Value::from("pattern");
    match mapping.get(&pattern_key) {
        Some(pattern) => match pattern {
            Value::String(string) => {
                Ok(Some(Regex::new(string).map_err(|e| {
                    Error::custom(format!("pattern `{:?}` is not a regex - {}", pattern, e))
                })?))
            }
            _ => Err(Error::custom(format!("pattern `{:#?}` must be a string", pattern))),
        },
        None => Ok(None),
    }
}

fn deserialize_constant_value<E>(mapping: &Mapping) -> Result<Option<EnumerationValue>, E>
where
    E: Error,
{
    let constant_key = Value::from("const");
    let value = mapping.get(&constant_key).map_or(Ok(None), |value| {
        serde_yaml::from_value(value.clone())
            .map_err(|e| Error::custom(format!("cannot deserialize constant specifier: {:?} - {}", value, e)))
    })?;
    let display_information = DisplayInformation {
        title: None,
        help: None,
        warning: None,
        description: None,
    };
    match value {
        None => Ok(None),
        Some(value) => Ok(Some(EnumerationValue {
            value,
            display_information,
        })),
    }
}

fn enumeration_definition_to_enumeration_value<E>(definition: &Value) -> Result<EnumerationValue, E>
where
    E: Error,
{
    match definition {
        Value::String(string) => Ok(string.as_str().into()),
        Value::Mapping(mapping) => Ok(mapping_to_enumeration_value(mapping)?),
        _ => Err(Error::custom(format!("no idea how to deserialize {:#?}", definition))),
    }
}

fn mapping_to_enumeration_value<E>(mapping: &Mapping) -> Result<EnumerationValue, E>
where
    E: Error,
{
    let value = mapping
        .get(&Value::from("value"))
        .map(|value| match value {
            Value::String(string) => Ok(string),
            _ => Err(Error::custom(format!("enum value `{:#?}` must be a string", value))),
        })
        .ok_or_else(|| Error::custom("when the enumeration is a mapping - expected 'value' to be present"))??;

    let title = mapping.get(&Value::from("title")).map(|value| match value {
        Value::String(string) => Ok(string),
        _ => Err(Error::custom(format!("enum title `{:#?}` must be a string", value))),
    });

    let title = match title {
        None => None,
        Some(result) => match result {
            Ok(value) => Some(value.to_string()),
            Err(e) => return Err(e),
        },
    };

    let display_information = DisplayInformation {
        title,
        help: None,
        warning: None,
        description: None,
    };
    Ok(EnumerationValue {
        display_information,
        value: value.to_string(),
    })
}

fn deserialize_integer_bound<E>(name: &str, mapping: &Mapping) -> Result<Option<IntegerBound>, E>
where
    E: Error,
{
    let normal = deserialize_integer(name, mapping)?;
    let exclusive = deserialize_integer(&("exclusive ".to_string() + name).to_mixed_case(), mapping)?;
    if normal.is_some() && exclusive.is_some() {
        return Err(Error::custom("cannot have both {} and exclusive {} set"));
    }
    if let Some(normal) = normal {
        return Ok(Some(IntegerBound::Inclusive(normal)));
    }
    if let Some(exclusive) = exclusive {
        return Ok(Some(IntegerBound::Exclusive(exclusive)));
    }
    Ok(None)
}