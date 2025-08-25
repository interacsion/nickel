use std::fmt;

use nickel_lang_core::{
    bytecode::ast::{
        document::{Document, Term},
        Ast,
    },
    identifier::LocIdent,
    term::Number,
};
use serde::{
    de::{
        self,
        value::{BorrowedStrDeserializer, MapDeserializer},
    },
    forward_to_deserialize_any,
};

pub use error::{Error, Result};

mod error;

pub struct Deserializer<'de> {
    fields: Vec<(&'de str, Value<'de>)>,
}

impl<'de> Deserializer<'de> {
    pub fn new(document: Document<'de>) -> Result<Self> {
        let mut fields = Vec::new();
        for field in document.field_defs {
            traverse_field(field.path, &field.value, &mut fields)?;
        }
        Ok(Self { fields })
    }
}

impl<'de> de::Deserializer<'de> for &'de Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(
            self.fields
                .iter()
                .map(|(k, v)| (BorrowedStrDeserializer::new(k), v)),
        ))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Debug)]
enum Value<'de> {
    Null,
    Bool(bool),
    Number(&'de Number),
    String(&'de str),
    EnumVariant(&'de str),
    Record(Vec<(&'de str, Value<'de>)>),
    Array(&'de [Term<'de>]),
    NickelTerm(&'de Ast<'de>),
}

impl<'de> TryFrom<&'de Term<'de>> for Value<'de> {
    type Error = Error;

    fn try_from(value: &'de Term<'de>) -> Result<Self> {
        Ok(match value {
            Term::Null => Value::Null,
            Term::Bool(b) => Value::Bool(*b),
            Term::Number(number) => Value::Number(number),
            Term::String(s) => Value::String(s),
            Term::EnumVariant(ident) => Value::EnumVariant(ident.label()),
            Term::Record(record) => {
                let mut fields = Vec::new();
                for field in record.field_defs {
                    traverse_field(field.path, &field.value, &mut fields)?;
                }
                Value::Record(fields)
            }
            Term::Array(array) => Value::Array(array),
            Term::NickelTerm(ast) => Value::NickelTerm(ast),
        })
    }
}

fn traverse_field<'de>(
    path: &'de [LocIdent],
    value: &'de Term,
    result: &mut Vec<(&'de str, Value<'de>)>,
) -> Result<()> {
    let key = path[0].label();
    let result_value = result.iter_mut().find(|(k, _)| *k == key);

    if path.len() == 1 {
        match result_value {
            Some((_, _v)) => {
                todo!()
            }
            None => result.push((key, value.try_into()?)),
        };
    } else {
        match result_value {
            Some((_, Value::Record(ref mut fields))) => traverse_field(&path[1..], value, fields)?,
            Some((_, _v)) => {
                todo!()
            }
            None => {
                let mut fields = Vec::new();
                traverse_field(&path[1..], value, &mut fields)?;
                result.push((key, Value::Record(fields)));
            }
        }
    }

    Ok(())
}

impl<'de> de::Deserializer<'de> for &'de Value<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Number(_number) => todo!(),
            Value::String(s) => visitor.visit_borrowed_str(s),
            Value::EnumVariant(s) => visitor.visit_borrowed_str(s),
            Value::Record(fields) => visitor.visit_map(MapDeserializer::new(
                fields
                    .iter()
                    .map(|(k, v)| (BorrowedStrDeserializer::new(k), v)),
            )),
            Value::Array(_field_defs) => todo!(),
            Value::NickelTerm(ast) => visitor.visit_map(MapDeserializer::new(std::iter::once((
                BorrowedStrDeserializer::new("$nickel-lang-document::private::NickelTerm"),
                *ast as *const Ast as usize as u64,
            )))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::IntoDeserializer<'de, Error> for &'de Value<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

#[derive(Clone, Debug)]
pub struct NickelTerm<'de> {
    pub ast: &'de Ast<'de>,
}

impl<'de> de::Deserialize<'de> for NickelTerm<'de> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = NickelTerm<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a nickel term")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                // TODO proper error handling
                let (k, v): (&str, u64) = map.next_entry()?.unwrap();
                assert_eq!(k, "$nickel-lang-document::private::NickelTerm");
                Ok(NickelTerm {
                    ast: unsafe { &*(v as usize as *const Ast) },
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}
