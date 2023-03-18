//! `lib.rs` - defines AST nodes and additional stuff.
pub mod location;
pub mod precedence;
pub mod token;

use std::collections::HashMap;

use location::{Span, WithSpan};
use token::Token;

/// Represents Ry source file.
#[derive(Debug, PartialEq)]
pub struct ProgramUnit {
    /// Global source file docstring
    pub docstring: String,

    pub imports: Vec<Import>,
    pub top_level_statements: Vec<(String, TopLevelStatement)>,
}

/// Import
///
/// ```ry
/// import std::io;
///        ------- `path`
/// ```
#[derive(Debug, PartialEq)]
pub struct Import {
    pub path: WithSpan<String>,
}

#[derive(Debug, PartialEq)]
pub enum TopLevelStatement {
    FunctionDecl(FunctionDecl),
    StructDecl(StructDecl),
    TraitDecl(TraitDecl),
    Impl(Impl),
    EnumDecl(EnumDecl),

    // just for better errors
    Import(Import),
}

/// Function declaration top level statement
///
/// ```ry
/// 1 | fun print_sum<T Number>(a T, b T) T
///   | ----------------------------------- `def`
/// 2 | {
///   |   ...
///   |   --- `stmts`
/// 7 | }
/// ```
#[derive(Debug, PartialEq)]
pub struct FunctionDecl {
    pub def: FunctionDef,
    pub stmts: Vec<Statement>,
}

pub type GenericAnnotation = (WithSpan<String>, Option<Type>);
pub type GenericAnnotations = Vec<GenericAnnotation>;

/// Function definition
///
/// ```ry
/// pub fun test<T Number, M, A>(a T, b T) T
/// ---     ---- --------------- --------  - `return_type`
/// |          | |                      |
/// `public`   | `generic_annotations`  |
///        `name`                      `params`
/// ```
#[derive(Debug, PartialEq)]
pub struct FunctionDef {
    pub public: Option<Span>,
    pub generic_annotations: GenericAnnotations,
    pub name: WithSpan<String>,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<Type>,
}

/// Struct declaration top level statement
///
/// ```ry
/// 1 | pub struct Test<B, C> {
///   | ---        ---- ----
///   | |          |       |
///   | `public`   |    `generic_annotations`
///   |            `name`
/// 2 |   /// documentation for the 1st member
///   |   ------------------------------------ `members.0.0`
/// 3 |   a B;
///   |   ---- `members.0.1`
/// 4 |
/// 5 |   ...
/// 6 | }
/// ```
#[derive(Debug, PartialEq)]
pub struct StructDecl {
    pub public: Option<Span>,
    pub generic_annotations: GenericAnnotations,
    pub name: WithSpan<String>,
    pub members: Vec<(String, StructMemberDef)>,
}

/// Trait implementation top level statement
///
/// ```ry
/// 1 | impl<A, B> Into<Tuple<A, B>> for Tuple<B, A> {
///   |     ------ -----------------     ----------- `type`
///   |     |                      |
///   |     |                 `trait`
///   |     `global_generic_annotations`
/// 2 |   ...
///   |   --- `methods`
/// 3 | }
/// ```
#[derive(Debug, PartialEq)]
pub struct Impl {
    pub public: Option<Span>,
    pub global_generic_annotations: GenericAnnotations,
    pub r#type: Type,
    pub r#trait: Option<Type>,
    pub methods: Vec<(String, TraitMethod)>,
}

/// Trait declaration top level statement
///
/// ```ry
/// 1 | pub trait Into<T> {
///   | ---       ---- - `generic_annotations`
///   | |            |
///   | `pub`    `name`
/// 2 |   ...
///   |   --- `methods`
/// 3 | }
/// ```
#[derive(Debug, PartialEq)]
pub struct TraitDecl {
    pub public: Option<Span>,
    pub name: WithSpan<String>,
    pub generic_annotations: GenericAnnotations,
    pub methods: Vec<(String, TraitMethod)>,
}

/// Trait method
///
/// ```ry
/// pub fun into<T>(self Self) T { ... }
/// ---     ---- -  ---------  -   --- `body`
/// |          | |          |  |
/// |          | |   `params` `return_type`
/// `public`   | `generic_annotations`
///        `name`
/// ```
#[derive(Debug, PartialEq)]
pub struct TraitMethod {
    pub public: Option<Span>,
    pub name: WithSpan<String>,
    pub generic_annotations: GenericAnnotations,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<Type>,
    pub body: Option<StatementsBlock>,
}

/// Enum declaration top level statement
///
/// ```ry
/// 1 | pub enum Test {
///   | ---      ---- `name`
///   | |
///   | `public`
///   |
/// 2 |   Test1,
///   |   ----- `variants.0.1`
/// 3 |   /// Some funny documentation
///   |   ---------------------------- `variants.1.0`
/// 4 |   Test2,
///   |   ----- `variants.1.1`
/// 5 | }
/// ```
#[derive(Debug, PartialEq)]
pub struct EnumDecl {
    pub public: Option<Span>,
    pub name: WithSpan<String>,
    pub variants: Vec<(String, WithSpan<String>)>,
}

/// ```ry
/// pub a [i32];
/// --- - ----- `type`
/// |   |
/// |   `name`
/// `public`
/// ```
#[derive(Debug, PartialEq)]
pub struct StructMemberDef {
    pub public: Option<Span>,
    pub name: WithSpan<String>,
    pub r#type: Type,
}

/// ```ry
/// pub fun test(a i32 = 0) {}
///              -^---^^^- function param
///              | |     |
///              | |     `default_value`
///              | `type`
///              `name`
/// ```
#[derive(Debug, PartialEq)]
pub struct FunctionParam {
    pub name: WithSpan<String>,
    pub r#type: Type,
    pub default_value: Option<Expression>,
}

pub type Type = WithSpan<Box<RawType>>;

#[derive(Debug, PartialEq)]
pub enum RawType {
    Array(Type),
    Pointer(Type),
    Primary(WithSpan<String>, Vec<Type>),
    Generic(WithSpan<String>),
    Option(Type),
}

pub type StatementsBlock = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(Expression),
    ExpressionWithoutSemicolon(Expression),
    Return(Expression),
    Defer(Expression),
    Var(WithSpan<String>, Option<Type>, Expression),
}

impl Statement {
    pub fn expression(self) -> Option<Expression> {
        match self {
            Self::Expression(e) => Some(e),
            _ => None,
        }
    }
}

pub type Expression = WithSpan<Box<RawExpression>>;

#[derive(Debug, PartialEq)]
pub enum RawExpression {
    String(String),
    Int(u64),
    Float(f64),
    Imag(f64),
    Bool(bool),
    Char(char),
    StaticName(String),
    List(Vec<Expression>),
    Binary(Expression, Token, Expression),
    As(Expression, Type),
    PrefixOrPostfix(Token, Expression),
    Property(Expression, WithSpan<String>),
    Struct(
        WithSpan<String>,
        HashMap<String, (Span, WithSpan<Expression>)>,
    ),
    Map(HashMap<String, (Span, WithSpan<Expression>)>),
    Call(Vec<Type>, Expression, Vec<Expression>),
    Index(Expression, Expression),
    If(
        (Expression, Vec<Statement>),
        Vec<(Expression, Vec<Statement>)>,
        Option<Vec<Statement>>,
    ),
    While(Expression, StatementsBlock),
}

impl RawExpression {
    pub fn must_have_semicolon_at_the_end(&self) -> bool {
        !matches!(
            self,
            RawExpression::If(_, _, _) | RawExpression::While(_, _)
        )
    }
}

pub trait WithSpannable {
    fn with_span(self, span: impl Into<Span>) -> WithSpan<Self>
    where
        Self: Sized;
}

impl<T: Sized> WithSpannable for T {
    fn with_span(self, span: impl Into<Span>) -> WithSpan<Self> {
        WithSpan::new(self, span.into())
    }
}
