use crate::term::Number;

use super::{Ast, LocIdent};

#[derive(Clone, Debug)]
pub struct Document<'ast> {
    pub field_defs: &'ast [FieldDef<'ast>],
}

#[derive(Clone, Debug)]
pub struct Record<'ast> {
    pub field_defs: &'ast [FieldDef<'ast>],
}

#[derive(Clone, Debug)]
pub struct FieldDef<'ast> {
    pub path: &'ast [LocIdent],
    pub value: Term<'ast>,
}

#[derive(Clone, Debug)]
pub enum Term<'ast> {
    Null,
    Bool(bool),
    Number(&'ast Number),
    String(&'ast str),
    EnumVariant(LocIdent),
    Record(Record<'ast>),
    Array(&'ast [Term<'ast>]),
    NickelTerm(&'ast Ast<'ast>),
}
