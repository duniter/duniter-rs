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

//! JSON parser based on [pest](https://pest.rs).  
//! It's is a personal crate for personal use.  
//! The grammar used is a copy of the grammar proposed in the "pest book".  

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
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
use std::str::FromStr;
use unwrap::unwrap;

#[derive(Parser)]
#[grammar = "json_grammar.pest"]
struct JSONParser;

#[derive(Debug, PartialEq)]
pub enum JSONValue<'a, S: std::hash::BuildHasher> {
    Object(HashMap<&'a str, JSONValue<'a, S>, S>),
    Array(Vec<JSONValue<'a, S>>),
    String(&'a str),
    Number(Number),
    Boolean(bool),
    Null,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Number {
    F64(f64),
    U64(u64),
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

    pub fn to_f64(&self) -> Option<f64> {
        if let JSONValue::Number(number) = self {
            match number {
                Number::F64(f64_) => Some(*f64_),
                Number::U64(u64_) => Some(*u64_ as f64),
            }
        } else {
            None
        }
    }

    pub fn to_u64(&self) -> Option<u64> {
        if let JSONValue::Number(number) = self {
            if let Number::U64(u64_) = number {
                Some(*u64_)
            } else {
                None
            }
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
            JSONValue::Number(n) => match n {
                Number::F64(f64_) => format!("{}", f64_),
                Number::U64(u64_) => format!("{}", u64_),
            },
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
        Ok(mut pair) => Ok(parse_value(unwrap!(
            pair.next(),
            "Fail to parse Rule::json"
        ))),
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
                    let name = unwrap!(
                        unwrap!(inner_rules.next(), "Fail to parse Rule::object::name")
                            .into_inner()
                            .next(),
                        "Fail to parse Rule::object::name"
                    )
                    .as_str();
                    let value = parse_value(unwrap!(
                        inner_rules.next(),
                        "Fail to parse Rule::object::value"
                    ));
                    (name, value)
                })
                .collect(),
        ),
        Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
        Rule::string => JSONValue::String(
            unwrap!(pair.into_inner().next(), "Fail to parse Rule::string").as_str(),
        ),
        Rule::number => {
            if let Ok(number_u64) = u64::from_str(pair.as_str()) {
                JSONValue::Number(Number::U64(number_u64))
            } else {
                JSONValue::Number(Number::F64(unwrap!(
                    pair.as_str().parse(),
                    "Fail to parse Rule::number as u64 and f64"
                )))
            }
        }
        Rule::boolean => JSONValue::Boolean(unwrap!(
            pair.as_str().parse(),
            "Fail to parse Rule::boolean"
        )),
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
                        .to_f64()
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

pub fn get_optional_str_not_empty<'a, S: std::hash::BuildHasher>(
    json_block: &'a HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<Option<&'a str>, Error> {
    let result = get_optional_str(json_block, field);
    if let Ok(Some(value)) = result {
        if !value.is_empty() {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    } else {
        result
    }
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

pub fn get_u64<S: std::hash::BuildHasher>(
    json_block: &HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<u64, Error> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must exist !", field),
        })?
        .to_u64()
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must be a number !", field),
        })?)
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
        .to_f64()
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

pub fn get_array<'a, S: std::hash::BuildHasher>(
    json_block: &'a HashMap<&str, JSONValue<S>, S>,
    field: &str,
) -> Result<Vec<&'a JSONValue<'a, S>>, ParseJsonError> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must exist !", field),
        })?
        .to_array()
        .ok_or_else(|| ParseJsonError {
            cause: format!("Fail to parse json : field '{}' must be an array !", field),
        })?
        .iter()
        .map(|v| v)
        .collect())
}

pub fn get_object_array<'a, S: std::hash::BuildHasher>(
    json_block: &'a JsonObject<'a, S>,
    field: &str,
) -> Result<Vec<&'a JsonObject<'a, S>>, ParseJsonError> {
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
            v.to_object().ok_or_else(|| ParseJsonError {
                cause: format!("Fail to parse json : field '{}' must be an object !", field),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_too_large_number() {
        assert_eq!(
            Ok(100_010_200_000_006_940),
            u64::from_str("100010200000006940"),
        );

        let json_string = "{
            \"nonce\": 100010200000006940
        }";

        let json_value = parse_json_string(json_string).expect("Fail to parse json string !");

        assert!(json_value.is_object());

        let json_object = json_value.to_object().expect("safe unwrap");

        assert_eq!(
            json_object.get("nonce"),
            Some(&JSONValue::Number(Number::U64(100_010_200_000_006_940)))
        );
    }

    #[test]
    fn test_parse_wrong_json_string() {
        let json_string = "{";
        assert!(parse_json_string(json_string).is_err());
    }

    #[test]
    fn test_parse_json_string() {
        let json_string = "{
            \"name\": \"toto\",
            \"age\": 25,
            \"legalAge\": true,
            \"ratio\": 0.5,
            \"friends\": [
                \"titi\",
                \"tata\"
            ],
            \"car\": null
        }";

        let json_value = parse_json_string(json_string).expect("Fail to parse json string !");

        assert_eq!(
            json_value.to_string(),
            "{\"name\":\"toto\",\"legalAge\":true,\"ratio\":0.5,\"age\":25,\"friends\":[\"titi\",\"tata\"],\"car\":null}"
        );

        assert!(json_value.is_object());
        assert!(!json_value.is_array());
        assert!(!json_value.is_str());
        assert!(!json_value.is_number());
        assert!(!json_value.is_bool());
        assert!(!json_value.is_null());
        assert_eq!(None, json_value.to_array());
        assert_eq!(None, json_value.to_str());
        assert_eq!(None, json_value.to_f64());
        assert_eq!(None, json_value.to_u64());
        assert_eq!(None, json_value.to_bool());

        let json_object = json_value.to_object().expect("safe unwrap");

        let name_field = json_object.get("name").expect("name field must be exist");
        assert!(name_field.is_str());
        assert_eq!(name_field, &JSONValue::String("toto"));

        let age_field = json_object.get("age").expect("age field must be exist");
        assert!(age_field.is_number());
        assert_eq!(age_field.to_f64(), Some(25.0f64));
        assert_eq!(age_field.to_u64(), Some(25u64));

        let legal_age_field = json_object
            .get("legalAge")
            .expect("legalAge field must be exist");
        assert!(legal_age_field.is_bool());
        assert_eq!(legal_age_field.to_bool(), Some(true));

        let ratio_field = json_object.get("ratio").expect("ratio field must be exist");
        assert!(ratio_field.is_number());
        assert_eq!(ratio_field.to_f64(), Some(0.5f64));
        assert_eq!(ratio_field.to_u64(), None);

        let friends_field = json_object
            .get("friends")
            .expect("friends field must be exist");
        assert!(!friends_field.is_object());
        assert_eq!(None, friends_field.to_object());
        assert!(friends_field.is_array());

        let friends = friends_field
            .to_array()
            .expect("frinds_field must be an array");

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

        let car_field = json_object.get("car").expect("car field must be exist");
        assert!(car_field.is_null());
    }
}
