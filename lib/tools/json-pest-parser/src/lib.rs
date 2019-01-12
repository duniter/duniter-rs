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
extern crate pest_derive;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "json_grammar.pest"]
struct JSONParser;

#[derive(Debug, PartialEq)]
pub enum JSONValue<'a> {
    Object(HashMap<&'a str, JSONValue<'a>>),
    Array(Vec<JSONValue<'a>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

impl<'a> JSONValue<'a> {
    pub fn is_object(&self) -> bool {
        if let JSONValue::Object(_) = self {
            true
        } else {
            false
        }
    }

    pub fn to_object(&self) -> Option<&HashMap<&'a str, JSONValue<'a>>> {
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

    pub fn to_array(&self) -> Option<&Vec<JSONValue<'a>>> {
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

impl<'a> ToString for JSONValue<'a> {
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

pub fn parse_json_string(source: &str) -> Result<JSONValue, Error<Rule>> {
    let json = JSONParser::parse(Rule::json, source)?.next().unwrap();

    Ok(parse_value(json))
}

fn parse_value(pair: Pair<Rule>) -> JSONValue {
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

        assert_eq!(
            JSONValue::Object(hashmap![
                "name" => JSONValue::String("toto"),
                "age" => JSONValue::Number(25f64),
                "friends" => JSONValue::Array(vec![JSONValue::String("titi"), JSONValue::String("tata"),])
            ]),
            json_value
        );

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
