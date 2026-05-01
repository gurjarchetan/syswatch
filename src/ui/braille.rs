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

/// Convert a slice of values into a Braille sparkline string.
///
/// * `data`   – historical samples (any numeric type mapped via u64)
/// * `width`  – desired output character width (each char = 2 data points)
/// * `height` – desired output row count    (each row = 4 data levels)
///
/// Returns a `Vec<String>` where index 0 is the TOP row.
pub fn render(data: &[u64], width: usize, height: usize) -> Vec<String> {
    if width == 0 || height == 0 || data.is_empty() {
        return vec![" ".repeat(width); height];
    }

    let total_cols = width * 2; // each braille char covers 2 data columns
    let total_rows = height * 4; // each braille char covers 4 data rows

    // Take the most recent `total_cols` samples; pad with 0 on the left.
    let padded: Vec<u64> = if data.len() >= total_cols {
        data[data.len() - total_cols..].to_vec()
    } else {
        let mut v = vec![0u64; total_cols - data.len()];
        v.extend_from_slice(data);
        v
    };

    let max_val = *padded.iter().max().unwrap_or(&1);
    let max_val = max_val.max(1);

    // Normalise to [0, total_rows]
    let normalised: Vec<usize> = padded
        .iter()
        .map(|&v| ((v as f64 / max_val as f64) * total_rows as f64).round() as usize)
        .collect();

    let mut rows: Vec<String> = Vec::with_capacity(height);

    for row_idx in 0..height {
        // row_idx 0 = top → braille rows 0..3 from the top
        // The TOP braille row corresponds to the HIGHEST data levels.
        let row_top = total_rows - row_idx * 4;      // inclusive top level
        let row_bot = row_top.saturating_sub(4);     // exclusive bottom level

        let mut line = String::with_capacity(width * 3);

        for col in 0..width {
            let left_val  = normalised[col * 2];
            let right_val = normalised[col * 2 + 1];

            let mut braille: u32 = BRAILLE_BASE;

            for (sample_val, col_idx) in [(left_val, 0usize), (right_val, 1usize)] {
                for dot_row in 0..4usize {
                    // dot_row 0 = topmost dot in the cell
                    let level = row_top - dot_row; // data level this dot represents
                    if level > row_bot && sample_val >= level {
                        braille |= DOT_MAP[col_idx][dot_row] as u32;
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
pub fn sparkline_f32(data: &[f32], width: usize, max_hint: f32) -> String {
    let max = if max_hint <= 0.0 { 100.0_f32 } else { max_hint };
    let u: Vec<u64> = data.iter().map(|&v| (v / max * 1000.0) as u64).collect();
    render(&u, width, 1).into_iter().next().unwrap_or_default()
}

/// Render a single-row sparkline from u64 values.
pub fn sparkline_u64(data: &[u64], width: usize) -> String {
    render(data, width, 1).into_iter().next().unwrap_or_default()
}
