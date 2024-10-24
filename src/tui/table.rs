use std::io::Result;

use crossterm::{
    style::{Print, PrintStyledContent, StyledContent, Stylize},
    QueueableCommand,
};

/// Table cells are a vector of styled content. We represent it as an array of StyledContent
/// rather than a string with ANSI codes so that we can easily calculate the displayed length
/// of the cell, without having to strip any non-visible codes.
#[derive(Clone)]
pub struct Cell {
    spans: Vec<StyledContent<String>>,
}

/// A Table consists of rows of cells, which can contain styled content.
#[derive(Clone)]
pub struct Table {
    pub width: usize,
    pub rows: Vec<Vec<Cell>>,
}

impl Cell {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<StyledContent<String>>,
    {
        Cell {
            spans: iter.into_iter().map(Into::into).collect(),
        }
    }

    pub fn plain<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Cell {
            spans: vec![s.into().stylize()],
        }
    }

    pub fn len(&self) -> usize {
        self.spans.iter().map(|s| s.content().chars().count()).sum()
    }
}

/// Queue a table for output. The table is truncated by its width, and columns are aligned.
pub fn queue_table(mut f: impl QueueableCommand, table: Table) -> Result<()> {
    // Calculate column widths
    let max_cols = table.rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut col_widths = vec![0; max_cols];
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    // Queue the padded cells, truncating to the width of the table
    const MIN_COL_WIDTH: usize = 3;
    const CELL_SPACING: usize = 2;
    for row in &table.rows {
        let mut pos = 0;
        for (i, cell) in row.iter().enumerate() {
            let col_width = col_widths[i] + CELL_SPACING;
            let cell_end_pos = (pos + col_width).min(table.width);
            let cell_width = cell_end_pos - pos;

            // If this cell has been truncated to less than MIN_COL_WIDTH, stop
            if col_width >= MIN_COL_WIDTH && cell_width < MIN_COL_WIDTH {
                break;
            };

            // Print all spans in the cell, truncating to the table width if needed
            for span in &cell.spans {
                let remaining_space = cell_end_pos - pos;
                let span_chars = span.content().chars();
                let span_length = span_chars.clone().count();

                if span_length > remaining_space {
                    let mut content: String = span_chars.take(remaining_space - 1).collect();
                    content.push('…');
                    f.queue(PrintStyledContent(StyledContent::new(
                        span.style().clone(),
                        content,
                    )))?;
                    pos += remaining_space;
                    break;
                } else {
                    f.queue(PrintStyledContent(span.clone()))?;
                    pos += span_length;
                }
            }

            // Print cell padding if needed
            if cell_end_pos > pos {
                let padding = cell_end_pos - pos;
                f.queue(Print(" ".repeat(padding)))?;
                pos += padding;
            }
        }
        f.queue(Print("\r\n"))?;
    }

    Ok(())
}
