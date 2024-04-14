pub fn span_offset_to_line_and_column(target_offset: usize, file_contents: &str) -> (usize, usize) {
    let mut line: usize = 1;
    let mut column: usize = 0;

    for (offset, char) in file_contents.char_indices() {
        if char == '\n' {
            line += 1;
            column = 0;
        }

        column += 1;

        if offset != target_offset {
            continue;
        }

        return (line, column - 1);
    }

    (0, 0)
}
