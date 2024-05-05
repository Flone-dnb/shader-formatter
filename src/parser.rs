use chumsky::{input::ValueInput, prelude::*};

pub type Span = SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Void,
    Bool,
    Integer,
    Float,
    Vector,
    Matrix,
    Texture,
    Sampler,
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
    Comment(&'src str),
    Other(char),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Groups parsed information about a struct.
#[derive(Clone, Debug, PartialEq)]
pub struct StructInfo<'src> {
    pub name: &'src str,
    pub fields: Vec<(Type, &'src str)>,
    pub docs: String,
}

/// Groups parsed information about a function argument.
#[derive(Clone, Debug, PartialEq)]
pub struct FuncArgument<'src> {
    pub _type: Type,
    pub name: &'src str,
    pub is_using_semantic: bool,
}

/// Groups parsed information about a function.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInfo<'src> {
    pub name: &'src str,
    pub args: Vec<FuncArgument<'src>>,
    pub return_type: Type,
    pub docs: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComplexToken<'src> {
    VariableDeclaration(Type, &'src str),
    Struct(StructInfo<'src>),
    Function(FunctionInfo<'src>),
    Other(Token<'src>),
}

impl std::fmt::Display for ComplexToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    // A parser for identifiers and keywords
    let ident = text::ascii::ident().map(|ident: &str| match ident {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "void" => Token::TypeName(Type::Void),
        "float" | "half" | "double" => Token::TypeName(Type::Float),
        "int" | "uint" | "dword" => Token::TypeName(Type::Integer),
        "bool" => Token::TypeName(Type::Bool),
        "float4" | "vec4" | "float2" | "vec2" | "float3" | "vec3" | "uint4" | "uvec4" | "uint3"
        | "uvec3" | "uint2" | "uvec2" => Token::TypeName(Type::Vector),
        "float4x4" | "mat4x4" | "float3x3" | "mat3x3" | "float2x2" | "mat2x2" => {
            Token::TypeName(Type::Matrix)
        }
        "Texture2D" | "sampler2D" => Token::TypeName(Type::Texture),
        "SamplerState" | "SamplerComparisonState" => Token::TypeName(Type::Sampler),
        _ => Token::Ident(ident),
    });

    // Parsers for comments.
    let simple_comment = just("//")
        .or(just("///"))
        .ignore_then(any().and_is(just("\n").not()).repeated().to_slice())
        .map(Token::Comment);
    let c_comment = just("/**")
        .or(just("/*!"))
        .ignore_then(any().and_is(just("*/").not()).repeated().to_slice())
        .then_ignore(just("*/"))
        .map(Token::Comment);
    let comment = c_comment.or(simple_comment);

    // A single token can be one of the above.
    let token = float
        .or(integer)
        .or(comment)
        .or(single_char_operator)
        .or(multi_char_operator)
        .or(ctrl)
        .or(ident)
        .or(any().map(Token::Other));

    token
        .map_with(|t, extra| (t, extra.span()))
        .padded()
        .repeated()
        .collect()
}

pub fn complex_token_parser<'src, I>(
) -> impl Parser<'src, I, Vec<(ComplexToken<'src>, Span)>, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    let var_type = select! { Token::TypeName(t) => t };
    let ident = select! { Token::Ident(ident) => ident };
    let comment = select! { Token::Comment(c) => c};
    let token = select! { token => token };

    // A parser for struct fields.
    let field = var_type
        .then(ident)
        .then_ignore(none_of(Token::Ctrl(';')).repeated())
        .then_ignore(just(Token::Ctrl(';')));

    // A parser for structs.
    let _struct = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(
            just(Token::Ident("struct"))
                .or(just(Token::Ident("uniform")))
                .or(just(Token::Ident("buffer"))),
        )
        .then(ident)
        .then_ignore(just(Token::Ctrl('{')))
        .then(field.repeated().collect())
        .then_ignore(just(Token::Ctrl('}')).or_not())
        .map(|((opt_comments, name), fields)| {
            ComplexToken::Struct(StructInfo {
                name,
                fields,
                docs: opt_comments.concat(),
            })
        });

    // A parser for variable declaration.
    let variable_declaration = var_type
        .then(ident)
        .then_ignore(just(Token::Op("=")).or_not())
        .then_ignore(none_of(Token::Ctrl(';')).repeated())
        .map(|(t, name)| ComplexToken::VariableDeclaration(t, name));

    // A parser for function arguments that use HLSL semantics.
    let argument_semantic = var_type
        .then(ident)
        .then_ignore(
            just(Token::Ctrl(':'))
                .then_ignore(ident)
                .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl(')')))),
        )
        .map(|(_type, name)| FuncArgument {
            _type,
            name,
            is_using_semantic: true,
        });

    // A parser for function arguments with custom (user) type.
    let custom_argument = ident
        .then(ident)
        .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl(')'))))
        .map(|(_type, name)| FuncArgument {
            _type: Type::Void, // use void for now
            name,
            is_using_semantic: false,
        });

    // A parser for function arguments with standard types.
    let standard_argument = var_type
        .then(ident)
        .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl(')'))))
        .map(|(_type, name)| FuncArgument {
            _type,
            name,
            is_using_semantic: false,
        });

    // A parser for function arguments.
    let argument = standard_argument.or(argument_semantic).or(custom_argument);

    // A parser for functions.
    let function = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then(var_type)
        .then(ident)
        .then_ignore(just(Token::Ctrl('(')))
        .then(argument.repeated().collect())
        .then_ignore(just(Token::Ctrl(')')).or_not())
        .map(|(((opt_comments, return_type), name), args)| {
            ComplexToken::Function(FunctionInfo {
                name,
                args,
                return_type,
                docs: opt_comments.concat(),
            })
        });

    // If non of our parsers from above worked then just pass the token.
    let output = _struct
        .or(function)
        .or(variable_declaration)
        .or(token.map(ComplexToken::Other));

    output
        .map_with(|t, extra| (t, extra.span()))
        .repeated()
        .collect()
}
