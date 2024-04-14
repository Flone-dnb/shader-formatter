use chumsky::{input::ValueInput, prelude::*};

pub type Span = SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Void,
    Bool,
    Integer,
    Float,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Op(&'src str),
    Ctrl(char),
    TypeName(Type),
    Ident(&'src str),
    Preprocessor(&'src str, &'src str),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'src> {
    VariableDeclaration(Type, &'src str),
    Token(Token<'src>),
}

pub fn statement_parser<'src, I>(
) -> impl Parser<'src, I, Vec<(Statement<'src>, Span)>, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    let var_type = select! { Token::TypeName(t) => t };
    let var_ident = select! { Token::Ident(ident) => ident };
    let token = select! { token => token };

    // A parser for variable declaration.
    let variable_declaration = var_type
        .then(var_ident)
        .then_ignore(just(Token::Op("=")))
        .map(|(t, name)| Statement::VariableDeclaration(t, name));

    // If non of our parsers from above worked then just pass the token.
    let output = variable_declaration.or(token.map(Statement::Token));

    output
        .map_with(|statement, extra| (statement, extra.span()))
        .repeated()
        .collect()
}

pub fn token_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    // A parser for integers
    let integer = just('-')
        .or_not()
        .then(text::int(10))
        .then_ignore(just('.').not())
        .to_slice()
        .map(|value: &str| Token::Integer(value.parse().unwrap()))
        .padded();

    // A parser for floats
    let float = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.'))
        .then(text::digits(10).or_not())
        .to_slice()
        .map(|value: &str| Token::Float(value.parse().unwrap()))
        .padded();

    // A parser for operators
    let single_char_operator = one_of("+*-/!=%|~&")
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| Token::Op(s));
    // some special cases that might conflic with other parsers
    let multi_char_operator = just(">=").or(just("<=")).map(|s: &str| Token::Op(s));

    // A parser for control characters (delimiters, semicolons, etc.)
    let ctrl = one_of("()[]{};,:<>.").map(Token::Ctrl);

    // A parser for preprocessor directives.
    let preprocessor = just('#')
        .ignore_then(
            any()
                .and_is(just(' ').not())
                .repeated()
                .to_slice()
                .padded()
                .then(any().and_is(just('\n').not()).repeated().to_slice())
                .map(|(keyword, value)| Token::Preprocessor(keyword, value)),
        )
        .padded();

    // A parser for identifiers and keywords
    let ident = text::ascii::ident().map(|ident: &str| match ident {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "void" => Token::TypeName(Type::Void),
        "float" | "half" | "double" => Token::TypeName(Type::Float),
        "int" | "uint" | "dword" => Token::TypeName(Type::Integer),
        "bool" => Token::TypeName(Type::Bool),
        _ => Token::Ident(ident),
    });

    // A single token can be one of the above.
    let token = preprocessor
        .or(float)
        .or(integer)
        .or(single_char_operator)
        .or(multi_char_operator)
        .or(ctrl)
        .or(ident);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded()
        .or(just("/**")
            .then(any().and_is(just("*/").not()).repeated())
            .padded());

    token
        .map_with(|t, extra| (t, extra.span()))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .collect()
}
