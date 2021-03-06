//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// Parsers for certifications
pub mod certifications;

/// Parsers for identities
pub mod identities;

/// Parsers for memberships
pub mod memberships;

/// Parsers for revocations
pub mod revoked;

/// Parsers for transactions
pub mod transactions;

use json_pest_parser::{JSONValue, Number};
use serde_json::Value;
use std::collections::HashMap;

/// Default hasher
pub type DefaultHasher = std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

#[derive(Copy, Clone, Debug, Fail)]
#[fail(display = "Fail to convert serde_json::Value into json_pest_parser::JSONValue")]
/// Error on conversion of serde_json value into pest_json value
pub struct JsonValueConversionError;

/// Convert serde_json value into pest_json value
pub fn serde_json_value_to_pest_json_value(
    value: &Value,
) -> Result<JSONValue<DefaultHasher>, JsonValueConversionError> {
    match value {
        Value::Null => Ok(JSONValue::Null),
        Value::Bool(boolean) => Ok(JSONValue::Boolean(*boolean)),
        Value::Number(number) => Ok(JSONValue::Number(if let Some(u64_) = number.as_u64() {
            Number::U64(u64_)
        } else if let Some(f64_) = number.as_f64() {
            Number::F64(f64_)
        } else {
            return Err(JsonValueConversionError);
        })),
        Value::String(string) => Ok(JSONValue::String(string)),
        Value::Array(values) => Ok(JSONValue::Array(
            values
                .iter()
                .map(serde_json_value_to_pest_json_value)
                .collect::<Result<Vec<JSONValue<DefaultHasher>>, JsonValueConversionError>>()?,
        )),
        Value::Object(map) => Ok(JSONValue::Object(
            map.into_iter()
                .map(|(k, v)| match serde_json_value_to_pest_json_value(v) {
                    Ok(v) => Ok((k.as_str(), v)),
                    Err(e) => Err(e),
                })
                .collect::<Result<
                    HashMap<&str, JSONValue<DefaultHasher>, DefaultHasher>,
                    JsonValueConversionError,
                >>()?,
        )),
    }
}

//std::collections::HashMap<&str, json_pest_parser::JSONValue<'_, std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>>>
//std::iter::Iterator<Item=(&std::string::String, json_pest_parser::JSONValue<'_, std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>>)>
