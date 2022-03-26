use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

pub struct CodeText {
    path: PathBuf,
    text: String,
    newlines: Vec<usize>,
}

impl CodeText {
    pub fn from_path(path: PathBuf) -> io::Result<CodeText> {
        let mut input_str = String::new();
        let mut f = File::open(&path)?;
        f.read_to_string(&mut input_str)?;
        Ok(CodeText::new(path, input_str))
    }

    pub fn new(path: PathBuf, text: String) -> CodeText {
        let newlines: Vec<usize> = {
            let input_indices = text
                .as_bytes()
                .iter()
                .enumerate()
                .filter(|&(_, &b)| b == b'\n')
                .map(|(i, _)| i + 1); // index of first char in the line
            Some(0).into_iter().chain(input_indices).collect()
        };
        CodeText {
            path,
            text,
            newlines,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    fn line_col(&self, pos: usize) -> (usize, usize) {
        let num_lines = self.newlines.len();
        let line = (0..num_lines)
            .filter(|&i| self.newlines[i] > pos)
            .map(|i| i - 1)
            .next()
            .unwrap_or(num_lines - 1);

        // offset of the first character in `line`
        let line_offset = self.newlines[line];

        // find the column; use `saturating_sub` in case `pos` is the
        // newline itself, which we'll call column 0
        let col = pos - line_offset;

        (line, col)
    }

    fn line_text(&self, line_num: usize) -> &str {
        let start_offset = self.newlines[line_num];
        if line_num == self.newlines.len() - 1 {
            &self.text[start_offset..]
        } else {
            let end_offset = self.newlines[line_num + 1];
            &self.text[start_offset..end_offset - 1]
        }
    }
}
