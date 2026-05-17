/// Braille Unicode block starts at U+2800.
/// Each cell encodes a 2×4 grid (2 wide, 4 tall).
/// Dot map:
///   col 0: dots 1(0x01), 2(0x02), 3(0x04), 7(0x40)
///   col 1: dots 4(0x08), 5(0x10), 6(0x20), 8(0x80)
const BRAILLE_BASE: u32 = 0x2800;

const DOT_MAP: [[u8; 4]; 2] = [
    [0x01, 0x02, 0x04, 0x40], // column 0 (left)
    [0x08, 0x10, 0x20, 0x80], // column 1 (right)
];

/// Convert a slice of u64 values into a Braille sparkline string.
///
/// Avoids all intermediate Vec allocations — works directly on the input slice.
/// Returns a `Vec<String>` where index 0 is the TOP row.
pub fn render(data: &[u64], width: usize, height: usize) -> Vec<String> {
    if width == 0 || height == 0 || data.is_empty() {
        return vec![" ".repeat(width); height];
    }

    let total_cols = width * 2; // each braille char covers 2 data columns
    let total_rows = height * 4; // each braille char covers 4 data rows
    let data_len = data.len();

    // Work on the last `total_cols` samples; pad with 0 on the left.
    let window_start = data_len.saturating_sub(total_cols);
    let window = &data[window_start..]; // len = min(data_len, total_cols)
    let padding = total_cols.saturating_sub(data_len); // leading zero columns

    let max_val = window.iter().max().copied().unwrap_or(1).max(1);

    // Map column index → normalised height — no heap allocation.
    let col_val = |col: usize| -> usize {
        if col < padding {
            return 0;
        }
        let v = window[col - padding];
        ((v as f64 / max_val as f64) * total_rows as f64).round() as usize
    };

    build_rows(width, height, total_rows, col_val)
}

/// Like `render()` but normalises against a fixed caller-supplied `max_val`.
/// Use for absolute scales (e.g. RAM% stored as `pct × 100`, pass `max_val = 10_000`).
pub fn render_absolute(data: &[u64], max_val: u64, width: usize, height: usize) -> Vec<String> {
    if width == 0 || height == 0 || data.is_empty() {
        return vec![" ".repeat(width); height];
    }

    let total_cols = width * 2;
    let total_rows = height * 4;
    let data_len = data.len();

    let window_start = data_len.saturating_sub(total_cols);
    let window = &data[window_start..];
    let padding = total_cols.saturating_sub(data_len);
    let max_v = max_val.max(1);

    let col_val = |col: usize| -> usize {
        if col < padding {
            return 0;
        }
        let v = window[col - padding];
        ((v as f64 / max_v as f64) * total_rows as f64)
            .round()
            .clamp(0.0, total_rows as f64) as usize
    };

    build_rows(width, height, total_rows, col_val)
}

/// Like `render()` but accepts `f32` values (e.g. CPU % history 0.0..100.0).
/// Avoids the caller needing to convert to `Vec<u64>` first.
pub fn render_f32(data: &[f32], width: usize, height: usize) -> Vec<String> {
    if width == 0 || height == 0 || data.is_empty() {
        return vec![" ".repeat(width); height];
    }

    let total_cols = width * 2;
    let total_rows = height * 4;
    let data_len = data.len();

    let window_start = data_len.saturating_sub(total_cols);
    let window = &data[window_start..];
    let padding = total_cols.saturating_sub(data_len);
    let max_val = window.iter().cloned().fold(0.0_f32, f32::max).max(1.0);

    let col_val = |col: usize| -> usize {
        if col < padding {
            return 0;
        }
        let v = window[col - padding];
        ((v / max_val) * total_rows as f32).round() as usize
    };

    build_rows(width, height, total_rows, col_val)
}

/// Shared row-building logic used by all render variants.
/// `col_val(col)` maps a column index (0..width*2) → normalised height.
#[inline]
fn build_rows(
    width: usize,
    height: usize,
    total_rows: usize,
    col_val: impl Fn(usize) -> usize,
) -> Vec<String> {
    let mut rows = Vec::with_capacity(height);

    for row_idx in 0..height {
        // row_idx 0 = top → braille rows covering the highest data levels.
        let row_top = total_rows - row_idx * 4; // inclusive top level
        let row_bot = row_top.saturating_sub(4); // exclusive bottom level

        let mut line = String::with_capacity(width * 3); // braille chars are 3 bytes each

        for col in 0..width {
            let left_val = col_val(col * 2);
            let right_val = col_val(col * 2 + 1);

            let mut braille: u32 = BRAILLE_BASE;

            for (sample_val, col_idx) in [(left_val, 0usize), (right_val, 1usize)] {
                for (dot_row, &dot_bit) in DOT_MAP[col_idx].iter().enumerate() {
                    // dot_row 0 = topmost dot in the cell
                    let level = row_top - dot_row; // data level this dot represents
                    if level > row_bot && sample_val >= level {
                        braille |= dot_bit as u32;
                    }
                }
            }

            line.push(char::from_u32(braille).unwrap_or(' '));
        }

        rows.push(line);
    }

    rows
}

/// Render a single-row sparkline from f32 values (0.0 .. max_hint).
#[allow(dead_code)]
pub fn sparkline_f32(data: &[f32], width: usize, max_hint: f32) -> String {
    let max = if max_hint <= 0.0 { 100.0_f32 } else { max_hint };
    let u: Vec<u64> = data.iter().map(|&v| (v / max * 1000.0) as u64).collect();
    render(&u, width, 1).into_iter().next().unwrap_or_default()
}

/// Render a single-row sparkline from u64 values.
#[allow(dead_code)]
pub fn sparkline_u64(data: &[u64], width: usize) -> String {
    render(data, width, 1)
        .into_iter()
        .next()
        .unwrap_or_default()
}
