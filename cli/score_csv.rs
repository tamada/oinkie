//! The score CSV: one file per compared pair, which `review` reads back.
//!
//! The format is the command line's, so it lives here rather than in the
//! library, which hands over a [`Comparison`] and the two things it compared
//! and has no opinion on how they are written down (#133).
//!
//! ```text
//! result,{nanoseconds},{similarity}
//! left,{the left side, as `Side::describe` writes it}
//! right,{the right side}
//! matrix,,{left function names...}
//! 0, {right function name},{score against each left function...}
//! ```

use oinkie::birthmarks::Birthmark;
use oinkie::compare::Comparison;
use oinkie::{Error, Program, Result};
use std::io::Write;
use std::path::Path;

/// One side of a comparison, as a line of the score CSV and as the names of
/// its functions.
pub(crate) trait Side {
    /// Everything after `left,` or `right,`.
    fn describe(&self) -> String;
    /// The function names, in the order the comparison's matrix uses.
    fn names(&self) -> Vec<String>;
}

fn json_path(path: Option<&Path>) -> String {
    path.map(|p| p.display().to_string()).unwrap_or_default()
}

impl Side for Birthmark {
    fn describe(&self) -> String {
        format!(
            "birthmark,{},{},{},{},{},{},{}",
            escape(self.name()),
            escape(&self.path().display().to_string()),
            self.birthmark_type(),
            self.extracted_at().to_rfc3339(),
            self.duration().as_nanos(),
            self.ir(),
            escape(&json_path(self.json_path()))
        )
    }

    fn names(&self) -> Vec<String> {
        self.elements()
            .iter()
            .map(|e| e.name().to_string())
            .collect()
    }
}

impl Side for Program {
    fn describe(&self) -> String {
        format!(
            "program,{},{},{},{},{}",
            escape(self.name()),
            escape(&self.path().display().to_string()),
            self.symbol_count(),
            self.len(),
            escape(&json_path(self.json_path()))
        )
    }

    fn names(&self) -> Vec<String> {
        self.function_names()
            .into_iter()
            .map(str::to_string)
            .collect()
    }
}

/// Writes one comparison to `dest`.
pub(crate) fn store<S: Side, P: AsRef<Path>>(comparison: &Comparison<S>, dest: P) -> Result<()> {
    let dest = dest.as_ref();
    let io_err = |e| Error::Io(dest.to_path_buf(), e);
    let mut file = std::fs::File::create(dest).map_err(io_err)?;
    let mut out = std::io::BufWriter::new(&mut file);
    writeln!(
        out,
        "result,{},{}",
        comparison.duration().as_nanos(),
        comparison.similarity()
    )
    .map_err(io_err)?;
    writeln!(out, "left,{}", comparison.columns().describe()).map_err(io_err)?;
    writeln!(out, "right,{}", comparison.rows().describe()).map_err(io_err)?;
    let column_names = comparison.columns().names();
    let row_names = comparison.rows().names();
    let header = column_names
        .iter()
        .map(|s| escape(s))
        .collect::<Vec<_>>()
        .join(",");
    write!(out, "matrix,,{header}").map_err(io_err)?;
    let matrix = comparison.matrix();
    for (j, item) in row_names.iter().enumerate() {
        write!(out, "\n{}, {}", j, escape(item)).map_err(io_err)?;
        for i in 0..column_names.len() {
            let value = matrix[[i, j]];
            write!(out, ",{value}").map_err(io_err)?;
        }
    }
    writeln!(out).map_err(io_err)?;
    out.flush().map_err(io_err)?;
    Ok(())
}

/// Quotes a value for embedding into a CSV field when it contains
/// characters that would otherwise break the record structure: a comma, a
/// quote, or either of the two characters that end a record. A carriage
/// return ends one as surely as a line feed, and both are legal in a Unix path
/// or a function name.
pub(crate) fn escape(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oinkie::birthmarks::BirthmarkType;
    use oinkie::compare::{Aggregator, Algorithm};
    use oinkie::extract::Extractor;
    use std::path::PathBuf;

    const CLANG: &str = "testdata/lifted/pcodes/hello_clang.json";
    const GCC: &str = "testdata/lifted/pcodes/hello_gcc.json";
    /// Three functions, against `CLANG`'s one: a matrix that is not square,
    /// so a transposed one cannot read the same.
    const UDL: &str = "testdata/lifted/pcodes/udl.json";
    /// One symbol and two functions: counts that differ, so swapping them in
    /// the line cannot read the same.
    const LLIL: &str = "testdata/lifted/bnil_llil/hello_clang.json";

    /// The fields of one line, split the way a CSV reader splits them.
    fn fields(line: &str) -> Vec<String> {
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.as_bytes())
            .records()
            .next()
            .unwrap()
            .unwrap()
            .iter()
            .map(str::to_string)
            .collect()
    }

    fn birthmark(lifted: &str) -> Birthmark {
        let program = Program::load(Path::new(lifted)).unwrap();
        Extractor::new(BirthmarkType::OpSeq)
            .extract(&program)
            .unwrap()
    }

    /// What is written is what `review` reads back: every score in its place,
    /// on a matrix that is not square, and the two sides' paths.
    #[test]
    fn test_a_comparison_is_read_back_whole_by_review() {
        let (b1, b2) = (birthmark(UDL), birthmark(CLANG));
        let c = Algorithm::Jaccard
            .comparator()
            .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian)
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("00000.csv");
        store(&c, &dest).unwrap();

        let content = std::fs::read_to_string(&dest).unwrap();
        assert!(content.starts_with("result,"), "{content}");
        let (loaded, left, right, _) = crate::review::load_comparison(&dest).unwrap();
        let written = c.matrix();
        let side = b1.len().max(b2.len());
        assert_eq!(written.dim(), (side, side), "padded to a square");
        assert_eq!(
            loaded.dim(),
            (b2.len(), b1.len()),
            "one row per right-hand function"
        );
        for i in 0..b1.len() {
            for j in 0..b2.len() {
                assert_eq!(loaded[[j, i]], written[[i, j]], "score [{i}, {j}]");
            }
        }
        assert!(
            (0..b1.len()).any(|i| written[[i, 0]] != written[[0, 0]]),
            "the fixture no longer tells a transposed matrix apart"
        );
        assert_eq!(left, b1.path());
        assert_eq!(right, b2.path());
    }

    /// A path with commas, quotes or a line break in it is quoted on the way
    /// out and comes back whole through the reader `review` uses. A carriage
    /// return ends a record as surely as a line feed does, and both are legal
    /// in a Unix path.
    #[test]
    fn test_a_path_that_needs_quoting_survives_the_round_trip() {
        for path in [
            "dir,with \"commas\"/app, v1",
            "dir/with\nline feed",
            "dir/with\rcarriage return",
        ] {
            let mut b1 = birthmark(CLANG);
            let b2 = birthmark(GCC);
            b1.metadata.path = PathBuf::from(path);
            let c = Algorithm::Jaccard
                .comparator()
                .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian)
                .unwrap();
            let dir = tempfile::tempdir().unwrap();
            let dest = dir.path().join("00000.csv");
            store(&c, &dest).unwrap();
            let (_, left, right, _) =
                crate::review::load_comparison(&dest).unwrap_or_else(|e| panic!("{path:?}: {e}"));
            assert_eq!(left, PathBuf::from(path), "{path:?}");
            assert_eq!(right, b2.path(), "{path:?}: the next record was disturbed");
        }
    }

    /// A program's line names it, counts its symbols and then its functions,
    /// and says where it was read from once that is known.
    #[test]
    fn test_a_program_describes_itself() {
        let mut p = Program::load(Path::new(LLIL)).unwrap();
        assert_ne!(
            p.symbol_count(),
            p.len(),
            "the fixture no longer tells the counts apart"
        );
        assert_eq!(
            fields(&p.describe()),
            [
                "program".to_string(),
                p.name().to_string(),
                p.path().display().to_string(),
                p.symbol_count().to_string(),
                p.len().to_string(),
                String::new(),
            ]
        );
        p.set_json_path(PathBuf::from(LLIL));
        assert_eq!(fields(&p.describe()).last().unwrap(), LLIL);
        assert_eq!(p.names().len(), p.len());
    }

    /// A birthmark's line gives each piece of what it is in a fixed order, and
    /// ends with where it was read from, empty until that is known.
    #[test]
    fn test_a_birthmark_describes_itself() {
        let mut b = birthmark(CLANG);
        assert_eq!(
            fields(&b.describe()),
            [
                "birthmark".to_string(),
                b.name().to_string(),
                b.path().display().to_string(),
                b.birthmark_type().to_string(),
                b.extracted_at().to_rfc3339(),
                b.duration().as_nanos().to_string(),
                b.ir().to_string(),
                String::new(),
            ]
        );
        b.set_json_path(PathBuf::from("birthmarks/hello_clang.json"));
        assert_eq!(
            fields(&b.describe()).last().unwrap(),
            "birthmarks/hello_clang.json"
        );
        assert_eq!(b.names().len(), b.len());
    }

    #[test]
    fn test_a_destination_that_cannot_be_written_is_an_io_error_on_it() {
        let b = birthmark(CLANG);
        let c = Algorithm::Jaccard
            .comparator()
            .compare_birthmarks(&b, &b, &Aggregator::Hungarian)
            .unwrap();
        let err = store(&c, "no/such/directory/out.csv").unwrap_err();
        assert!(
            matches!(&err, Error::Io(p, _) if p == Path::new("no/such/directory/out.csv")),
            "{err:?}"
        );
    }

    #[test]
    fn test_a_value_is_quoted_only_when_it_needs_to_be() {
        assert_eq!(escape("plain"), "plain");
        assert_eq!(escape("a,b"), "\"a,b\"");
        assert_eq!(escape("a\"b"), "\"a\"\"b\"");
        assert_eq!(escape("a\nb"), "\"a\nb\"");
        assert_eq!(escape("a\rb"), "\"a\rb\"");
    }
}
