#[derive(Clone, Copy, PartialEq)]
pub enum IndentationRule {
    Tab,
    TwoSpaces,
    FourSpaces,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NewLineOnOpenBrace {
    Before,
    After,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Case {
    Camel,
    Pascal,
    Snake,
    UpperSnake,
}
