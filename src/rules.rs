#[derive(Clone, Copy)]
pub enum IndentationRule {
    Tab,
    TwoSpaces,
    FourSpaces,
}

#[derive(Clone, Copy)]
pub enum NewLineAroundOpenBraceRule {
    Before,
    After,
}

#[derive(Clone, Copy)]
pub enum Case {
    Camel,
    Pascal,
    Snake,
    UpperSnake,
}
