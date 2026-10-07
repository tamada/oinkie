//! The assignment that pairs each function of one birthmark with at most one
//! function of the other so that the pairs' similarities sum to the most.
//!
//! The solver is the shortest-augmenting-path form of the Hungarian method.
//! It reads the similarities where they lie, row by row, and keeps only a
//! handful of vectors as long as the wider side: a pair's matrix is the
//! largest thing a comparison holds, and the solver must not need a second
//! one. The matrix may be rectangular, so the narrower side need not be
//! padded with zeros to make it square.

/// The pairing of rows with columns that maximises the sum of the paired
/// cells.
///
/// `cells` holds `rows * cols` similarities in row-major order. The result
/// holds, for each row, the column it is paired with; when there are more
/// rows than columns, `rows - cols` of them are paired with none.
///
/// `None` when a cell is not finite or `cells` is not `rows * cols` long:
/// a NaN compares false against everything, and no pairing maximises a sum
/// that contains one.
pub(crate) fn assign<T>(rows: usize, cols: usize, cells: &[T]) -> Option<Vec<Option<usize>>>
where
    T: Copy + Into<f64>,
{
    if cells.len() != rows.checked_mul(cols)? || !cells.iter().all(|&c| c.into().is_finite()) {
        return None;
    }
    Some(assign_by(rows, cols, |i, j| cells[i * cols + j].into()))
}

/// [`assign`] over values computed from a row and a column rather than read
/// from a slice, so that a pairing by values derived from a matrix -- its
/// cells weighted, say -- needs no second matrix to hold them.
///
/// `value` must be finite for every row and column; [`assign`] checks that
/// of its cells, and a caller deriving them from finite cells and finite
/// factors has it already.
pub(crate) fn assign_by(
    rows: usize,
    cols: usize,
    value: impl Fn(usize, usize) -> f64,
) -> Vec<Option<usize>> {
    if rows <= cols {
        solve(rows, cols, value)
    } else {
        // The method needs the narrower side as its rows, so the columns
        // take that role and the result is turned back around.
        let by_col = solve(cols, rows, |j, i| value(i, j));
        let mut by_row = vec![None; rows];
        for (j, i) in by_col.into_iter().enumerate() {
            if let Some(i) = i {
                by_row[i] = Some(j);
            }
        }
        by_row
    }
}

/// Pairs each of `n` rows with a distinct one of `m >= n` columns, maximising
/// the sum of `similarity(row, col)`.
///
/// Rows enter one at a time, and each is placed by a shortest path in the
/// reduced costs, so the potentials `u` and `v` stay feasible throughout and
/// the pairing is optimal once the last row is in. Index 0 of `p`, `way`,
/// `v`, `min_to` and `visited` is a virtual column that holds the row being
/// placed; the real columns are 1..=m, and `p[j] == 0` marks column `j` free.
fn solve(n: usize, m: usize, similarity: impl Fn(usize, usize) -> f64) -> Vec<Option<usize>> {
    // Maximising similarity is minimising its negation.
    let cost = |i: usize, j: usize| -similarity(i - 1, j - 1);
    let mut u = vec![0.0f64; n + 1];
    let mut v = vec![0.0f64; m + 1];
    let mut p = vec![0usize; m + 1];
    let mut way = vec![0usize; m + 1];
    let mut min_to = vec![f64::INFINITY; m + 1];
    let mut visited = vec![false; m + 1];
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0;
        min_to.fill(f64::INFINITY);
        visited.fill(false);
        loop {
            visited[j0] = true;
            let i0 = p[j0];
            let mut delta = f64::INFINITY;
            let mut j1 = 0;
            for j in 1..=m {
                if visited[j] {
                    continue;
                }
                let reduced = cost(i0, j) - u[i0] - v[j];
                if reduced < min_to[j] {
                    min_to[j] = reduced;
                    way[j] = j0;
                }
                if min_to[j] < delta {
                    delta = min_to[j];
                    j1 = j;
                }
            }
            for j in 0..=m {
                if visited[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    min_to[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        while j0 != 0 {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
        }
    }
    let mut by_row = vec![None; n];
    for j in 1..=m {
        if p[j] != 0 {
            by_row[p[j] - 1] = Some(j - 1);
        }
    }
    by_row
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic generator, so that a failing case can be
    /// reproduced from its seed.
    struct XorShift(u64);

    impl XorShift {
        fn next_f64(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn total(cells: &[f64], cols: usize, pairing: &[Option<usize>]) -> f64 {
        pairing
            .iter()
            .enumerate()
            .filter_map(|(i, j)| j.map(|j| cells[i * cols + j]))
            .sum()
    }

    /// The best total over every injective pairing of the narrower side into
    /// the wider.
    fn brute_force(rows: usize, cols: usize, cells: &[f64]) -> f64 {
        fn walk(i: usize, rows: usize, cols: usize, cells: &[f64], used: &mut Vec<bool>) -> f64 {
            if i == rows {
                return 0.0;
            }
            let mut best = if rows > cols {
                // With more rows than columns, a row may go unpaired.
                walk(i + 1, rows, cols, cells, used)
            } else {
                f64::NEG_INFINITY
            };
            for j in 0..cols {
                if !used[j] {
                    used[j] = true;
                    best = best.max(cells[i * cols + j] + walk(i + 1, rows, cols, cells, used));
                    used[j] = false;
                }
            }
            best
        }
        walk(0, rows, cols, cells, &mut vec![false; cols])
    }

    fn is_valid(rows: usize, cols: usize, pairing: &[Option<usize>]) -> bool {
        let mut seen = vec![false; cols];
        pairing.len() == rows
            && pairing
                .iter()
                .flatten()
                .all(|&j| !std::mem::replace(&mut seen[j], true))
            && pairing.iter().flatten().count() == rows.min(cols)
    }

    #[test]
    fn matches_brute_force_on_small_matrices() {
        let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
        for rows in 0..=6 {
            for cols in 0..=6 {
                for _ in 0..30 {
                    let cells: Vec<f64> = (0..rows * cols).map(|_| rng.next_f64()).collect();
                    let pairing = assign(rows, cols, &cells).unwrap();
                    assert!(is_valid(rows, cols, &pairing), "{rows}x{cols}: {pairing:?}");
                    let (got, want) = (
                        total(&cells, cols, &pairing),
                        brute_force(rows, cols, &cells),
                    );
                    assert!((got - want).abs() < 1e-9, "{rows}x{cols}: {got} != {want}");
                }
            }
        }
    }

    #[test]
    fn matches_brute_force_when_cells_tie() {
        // Few distinct values, as zero-padded and identical functions give.
        let mut rng = XorShift(42);
        for n in 1..=6 {
            for _ in 0..50 {
                let cells: Vec<f64> = (0..n * n)
                    .map(|_| (rng.next_f64() * 3.0).floor() / 2.0)
                    .collect();
                let pairing = assign(n, n, &cells).unwrap();
                assert!(is_valid(n, n, &pairing));
                assert!((total(&cells, n, &pairing) - brute_force(n, n, &cells)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn matches_brute_force_on_one_larger_matrix() {
        // 8! pairings: large enough for several augmenting paths to cross
        let mut rng = XorShift(7);
        let cells: Vec<f64> = (0..64).map(|_| rng.next_f64()).collect();
        let pairing = assign(8, 8, &cells).unwrap();
        assert!(is_valid(8, 8, &pairing));
        assert!((total(&cells, 8, &pairing) - brute_force(8, 8, &cells)).abs() < 1e-9);
    }

    #[test]
    fn accepts_f32() {
        let cells = [0.1f32, 0.9, 0.8, 0.2];
        assert_eq!(assign(2, 2, &cells), Some(vec![Some(1), Some(0)]));
    }

    #[test]
    fn rejects_cells_that_are_not_finite_or_not_rows_by_cols() {
        assert_eq!(assign(2, 2, &[0.5, f64::NAN, 0.5, 0.5]), None);
        assert_eq!(assign(1, 2, &[f64::INFINITY, 0.5]), None);
        assert_eq!(assign(2, 2, &[0.5, 0.5, 0.5]), None);
    }

    #[test]
    fn an_empty_side_pairs_nothing() {
        assert_eq!(assign::<f64>(0, 3, &[]), Some(vec![]));
        assert_eq!(assign::<f64>(3, 0, &[]), Some(vec![None, None, None]));
    }
}
