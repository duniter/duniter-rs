//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Parse JSON String.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate pest_derive;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use failure::Error;
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "json_grammar.pest"]
struct JSONParser;

#[derive(Debug, PartialEq)]
pub enum JSONValue<'a, S: std::hash::BuildHasher> {
    Object(HashMap<&'a str, JSONValue<'a, S>, S>),
    Array(Vec<JSONValue<'a, S>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

type JsonObject<'a, S> = HashMap<&'a str, JSONValue<'a, S>, S>;

impl<'a, S: std::hash::BuildHasher> JSONValue<'a, S> {
    pub fn is_object(&self) -> bool {
        if let JSONValue::Object(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_object(&self) -> Option<&HashMap<&'a str, JSONValue<'a, S>, S>> {
        if let JSONValue::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    pub fn is_array(&self) -> bool {
        if let JSONValue::Array(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_array(&self) -> Option<&Vec<JSONValue<'a, S>>> {
        if let JSONValue::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    pub fn is_str(&self) -> bool {
        if let JSONValue::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_str(&self) -> Option<&'a str> {
        if let JSONValue::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn is_number(&self) -> bool {
        if let JSONValue::Number(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_number(&self) -> Option<f64> {
        if let JSONValue::Number(number) = self {
            Some(*number)
        } else {
            None
        }
    }

    pub fn is_bool(&self) -> bool {
        if let JSONValue::Boolean(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let JSONValue::Boolean(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn is_null(&self) -> bool {
        if let JSONValue::Null = self {
            true
        } else {
            false
        }
    }
}

impl<'a, S: std::hash::BuildHasher> ToString for JSONValue<'a, S> {
    fn to_string(&self) -> String {
        match self {
            JSONValue::Object(o) => {
                let contents: Vec<_> = o
                    .iter()
                    .map(|(name, value)| format!("\"{}\":{}", name, value.to_string()))
                    .collect();
                format!("{{{}}}", contents.join(","))
            }
            JSONValue::Array(a) => {
                let contents: Vec<_> = a.iter().map(Self::to_string).collect();
                format!("[{}]", contents.join(","))
            }
            JSONValue::String(s) => format!("\"{}\"", s),
            JSONValue::Number(n) => format!("{}", n),
            JSONValue::Boolean(b) => format!("{}", b),
            JSONValue::Null => "null".to_owned(),
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Fail to parse JSON String : {:?}", cause)]
pub struct ParseJsonError {
    pub cause: String,
}

pub fn parse_json_string<'a>(
    source: &'a str,
) -> Result<
    JSONValue<'a, std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>>,
    ParseJsonError,
> {
    parse_json_string_with_specific_hasher::<
        std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>,
    >(source)
}

pub fn parse_json_string_with_specific_hasher<S: std::hash::BuildHasher + Default>(
    source: &str,
) -> Result<JSONValue<S>, ParseJsonError> {
    match JSONParser::parse(Rule::json, source) {
        Ok(mut pair) => Ok(parse_value(pair.next().unwrap())),
        Err(pest_error) => Err(ParseJsonError {
            cause: format!("{:?}", pest_error),
        }),
    }
}

fn parse_value<S: std::hash::BuildHasher + Default>(pair: Pair<Rule>) -> JSONValue<S> {
    match pair.as_rule() {
        Rule::object => JSONValue::Object(
            pair.into_inner()
                .map(|pair| {
                    let mut inner_rules = pair.into_inner();
                    let name = inner_rules
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str();
                    let value = parse_value(inner_rules.next().unwrap());
                    (name, value)
                })
                .collect(),
        ),
        Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
        Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
        Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
        Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
        Rule::null => JSONValue::Null,
        Rule::json
        | Rule::EOI
        | Rule::pair
        | Rule::value
        | Rule::inner_string
        | Rule::char
        | Rule::WHITESPACE => unreachable!(),
    }
}

pub fn get_optional_usize<S: std::hash::BuildHasher>(
    json_block: &HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<Option<usize>, Error> {
    Ok(match json_block.get(field) {
        Some(value) => {
            if !value.is_null() {
                Some(
                    value
                        .to_number()
                        .ok_or_else(|| ParseJsonError {
                            cause: format!(
                                "Fail to parse json : field '{}' must be a number !",
                                field
                            ),
                        })?
                        .trunc() as usize,
                )
            } else {
                None
            }
        }
        None => None,
    })
}

pub fn get_optional_str<'a, S: std::hash::BuildHasher>(
    json_block: &'a HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<Option<&'a str>, Error> {
    Ok(match json_block.get(field) {
        Some(value) => {
            if !value.is_null() {
                Some(value.to_str().ok_or_else(|| ParseJsonError {
                    cause: format!("Fail to parse json : field '{}' must be a string !", field),
                })?)
            } else {
                None
            }
        }
        None => None,
    })
}

pub fn get_number<S: std::hash::BuildHasher>(
    json_block: &HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<f64, Error> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must exist !", field),
        })?
        .to_number()
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must be a number !", field),
        })?)
}

pub fn get_str<'a, S: std::hash::BuildHasher>(
    json_block: &'a HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<&'a str, Error> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must exist !", field),
        })?
        .to_str()
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must be a string !", field),
        })?)
}

pub fn get_str_array<'a, S: std::hash::BuildHasher>(
    json_block: &'a HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<Vec<&'a str>, ParseJsonError> {
    json_block
        .get(field)
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must exist !", field),
        })?
        .to_array()
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must be an array !", field),
        })?
        .iter()
        .map(|v| {
            v.to_str().ok_or_else(|| ParseJsonError {
                cause: format!(
                    "Fail to parse json : field '{}' must be an array of string !",
                    field
                ),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_string() {
        let json_string = "{
            \"name\": \"toto\",
            \"age\": 25,
            \"friends\": [
                \"titi\",
                \"tata\"
            ]
        }";

        let json_value = parse_json_string(json_string).expect("Fail to parse json string !");

        assert!(json_value.is_object());

        let json_object = json_value.to_object().expect("safe unwrap");

        assert_eq!(json_object.get("name"), Some(&JSONValue::String("toto")));
        assert_eq!(json_object.get("age"), Some(&JSONValue::Number(25f64)));

        let friends = json_object
            .get("friends")
            .expect("frinds field must be exist")
            .to_array()
            .expect("frinds field must be an array");

        assert_eq!(2, friends.len());
        assert_eq!(
            "titi",
            friends[0]
                .to_str()
                .expect("friends field must be an array of String")
        );
        assert_eq!(
            "tata",
            friends[1]
                .to_str()
                .expect("friends field must be an array of String")
        );
    }

}
