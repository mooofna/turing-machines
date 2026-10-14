//! Plain aligned tables for terminal output.

use std::io::{self, Write};

use anstyle::Style;

/// How a column aligns its cells.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Align {
    /// Text, flush left.
    Left,
    /// Numbers, flush right.
    Right,
}

/// One cell: text and an optional style that does not count towards the width.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cell {
    text: String,
    style: Style,
}

impl Cell {
    /// A cell with `text` in `style`.
    pub(crate) fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    fn width(&self) -> usize {
        self.text.chars().count()
    }
}

impl From<String> for Cell {
    fn from(text: String) -> Self {
        Self {
            text,
            style: Style::new(),
        }
    }
}

impl From<&str> for Cell {
    fn from(text: &str) -> Self {
        Self::from(text.to_owned())
    }
}

/// A table with a header row, columns separated by three spaces.
#[derive(Clone, Debug)]
pub(crate) struct Table {
    columns: Vec<(String, Align)>,
    rows: Vec<Vec<Cell>>,
    indent: usize,
}

impl Table {
    /// A table with the given column headers and alignments.
    pub(crate) fn new(columns: &[(&str, Align)]) -> Self {
        Self {
            columns: columns.iter().map(|&(h, a)| (h.to_owned(), a)).collect(),
            rows: Vec::new(),
            indent: 0,
        }
    }

    /// Indents every line by `spaces`.
    #[must_use]
    pub(crate) fn indent(mut self, spaces: usize) -> Self {
        self.indent = spaces;
        self
    }

    /// Appends a row; missing cells are left empty.
    pub(crate) fn row(&mut self, cells: impl IntoIterator<Item = Cell>) {
        self.rows.push(cells.into_iter().collect());
    }

    /// Writes the table. The header is bold.
    pub(crate) fn write(&self, out: &mut dyn Write) -> io::Result<()> {
        let widths: Vec<usize> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, (header, _))| {
                self.rows
                    .iter()
                    .filter_map(|r| r.get(i).map(Cell::width))
                    .chain([header.chars().count()])
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let header: Vec<Cell> = self
            .columns
            .iter()
            .map(|(h, _)| Cell::styled(h.clone(), Style::new().bold()))
            .collect();
        self.write_row(out, &header, &widths)?;
        for row in &self.rows {
            self.write_row(out, row, &widths)?;
        }
        Ok(())
    }

    fn write_row(&self, out: &mut dyn Write, row: &[Cell], widths: &[usize]) -> io::Result<()> {
        let mut line = " ".repeat(self.indent);
        let last = self.columns.len().saturating_sub(1);
        for (i, ((_, align), &width)) in self.columns.iter().zip(widths).enumerate() {
            let empty = Cell::default();
            let cell = row.get(i).unwrap_or(&empty);
            let pad = " ".repeat(width.saturating_sub(cell.width()));
            let text = format!("{}{}{:#}", cell.style, cell.text, cell.style);
            match align {
                Align::Left if i == last => line.push_str(&text),
                Align::Left => {
                    line.push_str(&text);
                    line.push_str(&pad);
                }
                Align::Right => {
                    line.push_str(&pad);
                    line.push_str(&text);
                }
            }
            if i != last {
                line.push_str("   ");
            }
        }
        writeln!(out, "{}", line.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligns_columns() {
        let mut t = Table::new(&[("name", Align::Left), ("count", Align::Right)]);
        t.row([Cell::from("a"), Cell::from("1,000")]);
        t.row([Cell::from("longer"), Cell::from("7")]);
        let mut buf = Vec::new();
        t.write(&mut buf).unwrap();
        let plain = String::from_utf8(buf).unwrap();
        let plain = plain.replace("\u{1b}[1m", "").replace("\u{1b}[0m", "");
        assert_eq!(plain, "name     count\na        1,000\nlonger       7\n");
    }
}
