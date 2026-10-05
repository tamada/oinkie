//! `oinkie stats`: what a set of birthmarks looks like, before comparing them.
//!
//! Two counts are kept apart by name throughout (#129). A birthmark file is one
//! program and holds one entry per **function**; each function's birthmark is
//! made of **elements** -- operations, calls or k-grams. "Birthmark length" is
//! the number of elements in one function's birthmark, which is the sense
//! #128's `--min-elements` uses. The library's types follow the same terms:
//! `Birthmark::functions()` holds the functions, and each `Function`'s `Data`
//! holds its elements.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::cli;
use oinkie::birthmarks::{Birthmark, BirthmarkType, Data, Kgram};
use oinkie::{Error, Result};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::Serialize;

pub(crate) fn perform(opts: cli::StatsOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    if opts.format() == cli::StatsFormat::Csv && opts.is_per_file() && opts.top().is_some() {
        return Err(Error::Parse(
            "a CSV file holds one table, so --per-file and --top cannot both be given with -f csv; use -f json or -f markdown for both".to_string(),
        ));
    }
    let paths = collect_paths(opts.inputs(), opts.is_recursive())?;
    let report = compute(&paths, opts.top());
    // The reason already names the file: every error loading a birthmark
    // carries its path.
    for s in &report.skipped {
        log::warn!("skipped, not readable as a birthmark: {}", s.reason);
    }
    let text = match opts.format() {
        cli::StatsFormat::Json => to_json(&report)?,
        cli::StatsFormat::Csv if opts.top().is_some() => to_csv_top(&report)?,
        cli::StatsFormat::Csv if opts.is_per_file() => to_csv_files(&report)?,
        cli::StatsFormat::Csv => to_csv_summary(&report)?,
        cli::StatsFormat::Markdown => to_markdown(&report, opts.is_per_file()),
    };
    match opts.dest() {
        Some(dest) => std::fs::write(dest, text).map_err(|e| Error::Io(dest.to_path_buf(), e))?,
        None => std::io::stdout()
            .write_all(text.as_bytes())
            .map_err(|e| Error::Io(PathBuf::from("<stdout>"), e))?,
    }
    Ok(vec![start.elapsed()])
}

/// The files the inputs name, in a fixed order.
///
/// A file given by name is taken whatever its extension, since the caller
/// chose it. A directory contributes only its `*.json`, because a directory of
/// birthmarks also tends to hold other things. Sorted, so that the per-file
/// rows and anything computed in order do not depend on the file system.
pub(crate) fn collect_paths(inputs: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for input in inputs {
        let meta = std::fs::metadata(input).map_err(|e| Error::Io(input.clone(), e))?;
        if meta.is_dir() {
            scan_directory(input, recursive, &mut paths)?;
        } else {
            paths.push(input.clone());
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn scan_directory(dir: &Path, recursive: bool, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(|e| Error::Io(dir.to_path_buf(), e))? {
        let path = entry.map_err(|e| Error::Io(dir.to_path_buf(), e))?.path();
        if path.is_dir() {
            if recursive {
                scan_directory(&path, recursive, paths)?;
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(())
}

/// Everything the command reports, in the shape the JSON output is written in.
#[derive(Debug, Serialize)]
pub(crate) struct Report {
    pub groups: Vec<GroupStats>,
    pub files: Vec<FileStats>,
    pub skipped: Vec<Skipped>,
}

/// A file that was given, or found, and is not a birthmark.
///
/// Reported rather than only logged, so that a summary never silently covers
/// fewer files than it was handed.
#[derive(Debug, Serialize)]
pub(crate) struct Skipped {
    pub path: PathBuf,
    pub reason: String,
}

/// One `(ir, birthmark_type)`: the birthmarks that could be compared with each
/// other.
#[derive(Debug, Serialize)]
pub(crate) struct GroupStats {
    pub ir: String,
    pub birthmark_type: String,
    pub files: usize,
    /// Functions per file.
    pub functions: Option<Summary>,
    /// Elements per function, over every function in the group.
    pub elements: Option<Summary>,
    /// Functions with no elements at all.
    pub empty: usize,
    /// `empty` over the number of functions; absent when there are none.
    pub empty_ratio: Option<f64>,
    /// Distinct elements across the group.
    pub vocabulary: usize,
    /// The total of the counts, for the `freq` shapes only: in the others an
    /// element's occurrences are already what `elements` counts, or are not
    /// recorded at all.
    pub occurrences: Option<usize>,
    /// The most frequent elements, present when `--top` was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<Vec<TopElement>>,
}

/// An element and how often it occurs in the group.
///
/// "How often" is what the shape records: occurrences for `seq` and `freq`,
/// and for `set`, the number of functions that contain it.
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct TopElement {
    pub element: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct FileStats {
    pub path: PathBuf,
    pub file_name: String,
    pub ir: String,
    pub birthmark_type: String,
    pub functions: usize,
    pub empty: usize,
    /// Elements per function in this file; absent when it has no functions.
    pub elements: Option<Summary>,
}

/// Descriptive statistics of a list of counts.
///
/// The standard deviation is the population one: the input is every function
/// in the set that was given, not a sample drawn from some larger one.
/// Quartiles interpolate linearly between ranks, as R's default and NumPy's do.
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct Summary {
    pub min: usize,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: usize,
    pub mean: f64,
    pub stddev: f64,
}

impl Summary {
    pub(crate) fn of(values: &[usize]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let n = sorted.len() as f64;
        let mean = sorted.iter().sum::<usize>() as f64 / n;
        let variance = sorted
            .iter()
            .map(|&v| (v as f64 - mean).powi(2))
            .sum::<f64>()
            / n;
        Some(Self {
            min: sorted[0],
            q1: quantile(&sorted, 0.25),
            median: quantile(&sorted, 0.5),
            q3: quantile(&sorted, 0.75),
            max: sorted[sorted.len() - 1],
            mean,
            stddev: variance.sqrt(),
        })
    }
}

fn quantile(sorted: &[usize], q: f64) -> f64 {
    let pos = (sorted.len() - 1) as f64 * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let frac = pos - lo as f64;
    sorted[lo] as f64 + (sorted[hi] as f64 - sorted[lo] as f64) * frac
}

/// What one birthmark file contributes, so that the files can be read in
/// parallel and folded together afterwards.
struct FileData {
    stats: FileStats,
    lengths: Vec<usize>,
    counts: FxHashMap<String, usize>,
    occurrences: Option<usize>,
}

pub(crate) fn compute(paths: &[PathBuf], top: Option<usize>) -> Report {
    let loaded = paths
        .par_iter()
        .map(|path| Birthmark::try_from(path.as_path()).map(|b| read_file(path, &b)))
        .collect::<Vec<_>>();

    let mut skipped = Vec::new();
    let mut files = Vec::new();
    let mut groups: BTreeMap<(String, String), Vec<FileData>> = BTreeMap::new();
    for (path, result) in paths.iter().zip(loaded) {
        match result {
            Ok(data) => {
                let key = (data.stats.ir.clone(), data.stats.birthmark_type.clone());
                groups.entry(key).or_default().push(data);
            }
            Err(e) => skipped.push(Skipped {
                path: path.clone(),
                reason: e.to_string(),
            }),
        }
    }
    let groups = groups
        .into_iter()
        .map(|((ir, birthmark_type), datas)| {
            let group = fold_group(ir, birthmark_type, &datas, top);
            files.extend(datas.into_iter().map(|d| d.stats));
            group
        })
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Report {
        groups,
        files,
        skipped,
    }
}

fn read_file(path: &Path, birthmark: &Birthmark) -> FileData {
    let lengths = birthmark
        .functions()
        .iter()
        .map(|f| f.len())
        .collect::<Vec<_>>();
    let mut counts = FxHashMap::default();
    let mut occurrences = None;
    for function in birthmark.functions() {
        match function.data() {
            Data::Seq(seq) => seq.iter().for_each(|e| add(&mut counts, e.clone(), 1)),
            Data::Set(set) => set.iter().for_each(|e| add(&mut counts, e.clone(), 1)),
            Data::Freq(freq) => {
                for (e, &n) in freq {
                    add(&mut counts, e.clone(), n);
                    *occurrences.get_or_insert(0) += n;
                }
            }
            Data::KgramSeq(seq) => seq.iter().for_each(|k| add(&mut counts, kgram(k), 1)),
            Data::KgramSet(set) => set.iter().for_each(|k| add(&mut counts, kgram(k), 1)),
            Data::KgramFreq(freq) => {
                for (k, &n) in freq {
                    add(&mut counts, kgram(k), n);
                    *occurrences.get_or_insert(0) += n;
                }
            }
        }
    }
    // A freq birthmark with no functions still has a total, and it is zero.
    if occurrences.is_none() && is_freq(birthmark.birthmark_type()) {
        occurrences = Some(0);
    }
    FileData {
        stats: FileStats {
            path: path.to_path_buf(),
            file_name: birthmark.name().to_string(),
            ir: birthmark.ir().to_string(),
            birthmark_type: birthmark.birthmark_type().to_string(),
            functions: lengths.len(),
            empty: lengths.iter().filter(|&&n| n == 0).count(),
            elements: Summary::of(&lengths),
        },
        lengths,
        counts,
        occurrences,
    }
}

fn is_freq(bt: &BirthmarkType) -> bool {
    matches!(
        bt,
        BirthmarkType::FcFreq | BirthmarkType::OpFreq | BirthmarkType::OpKgramFreq(_)
    )
}

fn add(counts: &mut FxHashMap<String, usize>, element: String, n: usize) {
    *counts.entry(element).or_insert(0) += n;
}

/// A k-gram as one string: its operations joined by spaces. Only operations
/// are ever k-grammed, and no mnemonic contains a space.
fn kgram(k: &Kgram) -> String {
    k.ops().join(" ")
}

fn fold_group(
    ir: String,
    birthmark_type: String,
    datas: &[FileData],
    top: Option<usize>,
) -> GroupStats {
    let per_file = datas.iter().map(|d| d.stats.functions).collect::<Vec<_>>();
    let lengths = datas
        .iter()
        .flat_map(|d| d.lengths.iter().copied())
        .collect::<Vec<_>>();
    let mut counts: FxHashMap<&str, usize> = FxHashMap::default();
    for d in datas {
        for (e, &n) in &d.counts {
            *counts.entry(e.as_str()).or_insert(0) += n;
        }
    }
    let empty = lengths.iter().filter(|&&n| n == 0).count();
    let occurrences = datas
        .iter()
        .map(|d| d.occurrences)
        .try_fold(0, |acc, o| o.map(|n| acc + n));
    let top = top.map(|n| {
        let mut ranked = counts.iter().collect::<Vec<_>>();
        // Most frequent first; ties by name, so the list is the same every run.
        ranked.sort_unstable_by(|(a, x), (b, y)| y.cmp(x).then(a.cmp(b)));
        ranked
            .into_iter()
            .take(n)
            .map(|(e, &count)| TopElement {
                element: e.to_string(),
                count,
            })
            .collect()
    });
    GroupStats {
        ir,
        birthmark_type,
        files: datas.len(),
        functions: Summary::of(&per_file),
        elements: Summary::of(&lengths),
        empty,
        empty_ratio: (!lengths.is_empty()).then(|| empty as f64 / lengths.len() as f64),
        vocabulary: counts.len(),
        occurrences,
        top,
    }
}

pub(crate) fn to_json(report: &Report) -> Result<String> {
    serde_json::to_string_pretty(report)
        .map(|s| s + "\n")
        .map_err(|e| Error::Json(PathBuf::from("<stats>"), e))
}

fn csv_string(rows: Vec<Vec<String>>) -> Result<String> {
    let mut w = csv::Writer::from_writer(Vec::new());
    for row in rows {
        w.write_record(row).map_err(Error::Csv)?;
    }
    let bytes = w
        .into_inner()
        .map_err(|e| Error::Parse(format!("could not write the CSV: {e}")))?;
    String::from_utf8(bytes).map_err(|e| Error::Parse(format!("could not write the CSV: {e}")))
}

const SUMMARY_FIELDS: [&str; 7] = ["min", "q1", "median", "q3", "max", "mean", "stddev"];

fn summary_cells(s: &Option<Summary>) -> Vec<String> {
    match s {
        Some(s) => vec![
            s.min.to_string(),
            s.q1.to_string(),
            s.median.to_string(),
            s.q3.to_string(),
            s.max.to_string(),
            s.mean.to_string(),
            s.stddev.to_string(),
        ],
        None => vec![String::new(); SUMMARY_FIELDS.len()],
    }
}

fn prefixed(prefix: &str) -> impl Iterator<Item = String> + '_ {
    SUMMARY_FIELDS.iter().map(move |f| format!("{prefix}_{f}"))
}

fn opt<T: ToString>(v: &Option<T>) -> String {
    v.as_ref().map(|v| v.to_string()).unwrap_or_default()
}

fn to_csv_summary(report: &Report) -> Result<String> {
    let mut header = vec!["ir".to_string(), "birthmark_type".into(), "files".into()];
    header.extend(prefixed("functions"));
    header.extend(prefixed("elements"));
    header.extend(["empty", "empty_ratio", "vocabulary", "occurrences"].map(String::from));
    let mut rows = vec![header];
    for g in &report.groups {
        let mut row = vec![g.ir.clone(), g.birthmark_type.clone(), g.files.to_string()];
        row.extend(summary_cells(&g.functions));
        row.extend(summary_cells(&g.elements));
        row.extend([
            g.empty.to_string(),
            opt(&g.empty_ratio),
            g.vocabulary.to_string(),
            opt(&g.occurrences),
        ]);
        rows.push(row);
    }
    csv_string(rows)
}

fn to_csv_files(report: &Report) -> Result<String> {
    let mut header = [
        "path",
        "file_name",
        "ir",
        "birthmark_type",
        "functions",
        "empty",
    ]
    .map(String::from)
    .to_vec();
    header.extend(prefixed("elements"));
    let mut rows = vec![header];
    for f in &report.files {
        let mut row = vec![
            f.path.display().to_string(),
            f.file_name.clone(),
            f.ir.clone(),
            f.birthmark_type.clone(),
            f.functions.to_string(),
            f.empty.to_string(),
        ];
        row.extend(summary_cells(&f.elements));
        rows.push(row);
    }
    csv_string(rows)
}

fn to_csv_top(report: &Report) -> Result<String> {
    let header = ["ir", "birthmark_type", "rank", "element", "count"]
        .map(String::from)
        .to_vec();
    let mut rows = vec![header];
    for g in &report.groups {
        for (i, t) in g.top.iter().flatten().enumerate() {
            rows.push(vec![
                g.ir.clone(),
                g.birthmark_type.clone(),
                (i + 1).to_string(),
                t.element.clone(),
                t.count.to_string(),
            ]);
        }
    }
    csv_string(rows)
}

/// Two decimals: this is the table read by a person, and the JSON and CSV
/// keep the full precision.
fn fixed(v: f64) -> String {
    format!("{v:.2}")
}

/// A cell of a Markdown table. A `|` would end the cell early, and a function
/// call name can in principle hold one.
fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

fn md_row(cells: &[String]) -> String {
    format!("| {} |\n", cells.join(" | "))
}

fn md_table(header: &[&str], right_from: usize, rows: &[Vec<String>]) -> String {
    let mut s = md_row(&header.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    let rule = (0..header.len())
        .map(|i| if i < right_from { "---" } else { "---:" }.to_string())
        .collect::<Vec<_>>();
    s.push_str(&md_row(&rule));
    rows.iter().for_each(|r| s.push_str(&md_row(r)));
    s
}

fn md_summary(s: &Option<Summary>) -> Vec<String> {
    match s {
        Some(s) => vec![
            s.min.to_string(),
            fixed(s.q1),
            fixed(s.median),
            fixed(s.q3),
            s.max.to_string(),
            fixed(s.mean),
            fixed(s.stddev),
        ],
        None => vec!["–".to_string(); SUMMARY_FIELDS.len()],
    }
}

fn to_markdown(report: &Report, per_file: bool) -> String {
    let mut s = String::from("# Birthmark statistics\n\n");
    s.push_str(&format!(
        "{} birthmark files in {} groups.",
        report.files.len(),
        report.groups.len()
    ));
    if !report.skipped.is_empty() {
        s.push_str(&format!(
            " {} files skipped, not readable as birthmarks.",
            report.skipped.len()
        ));
    }
    s.push_str(
        "\n\nA birthmark file holds one entry per **function**; each function's birthmark \
         is made of **elements**. *Elements per function* is the birthmark length.\n",
    );

    s.push_str("\n## Functions per file\n\n");
    let rows = report
        .groups
        .iter()
        .map(|g| {
            let mut r = vec![cell(&g.ir), cell(&g.birthmark_type), g.files.to_string()];
            r.extend(md_summary(&g.functions));
            r
        })
        .collect::<Vec<_>>();
    let header = [
        "ir",
        "birthmark",
        "files",
        "min",
        "Q1",
        "median",
        "Q3",
        "max",
        "mean",
        "sd",
    ];
    s.push_str(&md_table(&header, 2, &rows));

    s.push_str("\n## Elements per function\n\n");
    let rows = report
        .groups
        .iter()
        .map(|g| {
            let mut r = vec![cell(&g.ir), cell(&g.birthmark_type)];
            r.extend(md_summary(&g.elements));
            r.extend([
                g.empty.to_string(),
                g.empty_ratio
                    .map(|v| format!("{:.1}%", v * 100.0))
                    .unwrap_or_else(|| "–".to_string()),
                g.vocabulary.to_string(),
                g.occurrences
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "–".to_string()),
            ]);
            r
        })
        .collect::<Vec<_>>();
    let header = [
        "ir",
        "birthmark",
        "min",
        "Q1",
        "median",
        "Q3",
        "max",
        "mean",
        "sd",
        "empty",
        "empty %",
        "vocabulary",
        "occurrences",
    ];
    s.push_str(&md_table(&header, 2, &rows));

    for g in &report.groups {
        let Some(top) = &g.top else { continue };
        s.push_str(&format!(
            "\n## Top {} elements: {} / {}\n\n",
            top.len(),
            cell(&g.ir),
            cell(&g.birthmark_type)
        ));
        let rows = top
            .iter()
            .enumerate()
            .map(|(i, t)| {
                vec![
                    (i + 1).to_string(),
                    format!("`{}`", cell(&t.element)),
                    t.count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        s.push_str(&md_table(&["rank", "element", "count"], 2, &rows));
    }

    if per_file {
        s.push_str("\n## Per file\n\n");
        let rows = report
            .files
            .iter()
            .map(|f| {
                let mut r = vec![
                    cell(&f.path.display().to_string()),
                    cell(&f.ir),
                    cell(&f.birthmark_type),
                    f.functions.to_string(),
                    f.empty.to_string(),
                ];
                let e = md_summary(&f.elements);
                // min, median, max, mean: the quartiles and sd make the row
                // too wide to read, and the JSON and CSV carry them.
                r.extend([e[0].clone(), e[2].clone(), e[4].clone(), e[5].clone()]);
                r
            })
            .collect::<Vec<_>>();
        let header = [
            "file",
            "ir",
            "birthmark",
            "functions",
            "empty",
            "elements min",
            "median",
            "max",
            "mean",
        ];
        s.push_str(&md_table(&header, 3, &rows));
    }

    if !report.skipped.is_empty() {
        s.push_str("\n## Skipped\n\n");
        for sk in &report.skipped {
            s.push_str(&format!("- {}\n", sk.reason));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use oinkie::lift::Ir;

    fn s(v: &str) -> String {
        v.to_string()
    }

    /// One function's birthmark, as the JSON a birthmark file holds for it.
    fn function(name: &str, data: Data) -> serde_json::Value {
        serde_json::json!({ "name": name, "data": data })
    }

    /// A birthmark built the way a reader of birthmark files gets one: from
    /// its JSON. Its fields are not public, and a file is what this command
    /// reads anyway.
    fn birthmark(name: &str, bt: BirthmarkType, functions: Vec<serde_json::Value>) -> Birthmark {
        serde_json::from_value(serde_json::json!({
            "metadata": {
                "file_name": name,
                "path": name,
                "extracted_at": chrono::Utc::now(),
                "duration": 1,
                "birthmark_type": bt,
                "ir": Ir::GhidraPcode,
            },
            "elements": functions,
        }))
        .expect("a birthmark built for a test should read")
    }

    fn write(dir: &Path, file: &str, b: &Birthmark) -> PathBuf {
        let path = dir.join(file);
        std::fs::write(&path, serde_json::to_string(b).unwrap()).unwrap();
        path
    }

    fn seq(ops: &[&str]) -> Data {
        Data::Seq(ops.iter().map(|o| s(o)).collect())
    }

    #[test]
    fn test_summary_matches_the_textbook_values() {
        // Worked by hand: mean 5, population variance 4, so sd 2; quartiles
        // at ranks 1.75, 3.5 and 5.25 of eight.
        let got = Summary::of(&[2, 4, 4, 4, 5, 5, 7, 9]).unwrap();
        assert_eq!(got.min, 2);
        assert_eq!(got.max, 9);
        assert_eq!(got.mean, 5.0);
        assert_eq!(got.stddev, 2.0);
        assert_eq!(got.q1, 4.0);
        assert_eq!(got.median, 4.5);
        assert_eq!(got.q3, 5.5);
    }

    #[test]
    fn test_summary_of_one_value_and_of_none() {
        let one = Summary::of(&[3]).unwrap();
        assert_eq!(
            (one.min, one.q1, one.median, one.q3, one.max),
            (3, 3.0, 3.0, 3.0, 3)
        );
        assert_eq!(one.stddev, 0.0);
        assert_eq!(Summary::of(&[]), None);
    }

    #[test]
    fn test_a_directory_gives_its_json_and_descends_only_when_asked() {
        let d = tempfile::tempdir().unwrap();
        let sub = d.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        for f in ["b.json", "a.json", "notes.txt"] {
            std::fs::write(d.path().join(f), "").unwrap();
        }
        std::fs::write(sub.join("c.json"), "").unwrap();

        let flat = collect_paths(&[d.path().to_path_buf()], false).unwrap();
        assert_eq!(flat, vec![d.path().join("a.json"), d.path().join("b.json")]);

        let deep = collect_paths(&[d.path().to_path_buf()], true).unwrap();
        assert_eq!(deep.len(), 3);
        assert!(deep.contains(&sub.join("c.json")));
    }

    #[test]
    fn test_a_file_named_directly_is_taken_whatever_its_extension() {
        let d = tempfile::tempdir().unwrap();
        let txt = d.path().join("notes.txt");
        std::fs::write(&txt, "").unwrap();
        // Named twice, once directly and once through its directory: read once.
        let json = d.path().join("a.json");
        std::fs::write(&json, "").unwrap();
        let got =
            collect_paths(&[txt.clone(), json.clone(), d.path().to_path_buf()], false).unwrap();
        assert_eq!(got, vec![json, txt]);
    }

    #[test]
    fn test_a_path_that_is_not_there_is_an_error_not_a_skip() {
        let d = tempfile::tempdir().unwrap();
        let missing = d.path().join("missing");
        assert!(matches!(
            collect_paths(&[missing], false),
            Err(Error::Io(_, _))
        ));
    }

    #[test]
    fn test_functions_and_elements_are_counted_apart() {
        let d = tempfile::tempdir().unwrap();
        let a = write(
            d.path(),
            "a.json",
            &birthmark(
                "a",
                BirthmarkType::OpSeq,
                vec![
                    function("f", seq(&["COPY", "COPY", "CALL"])),
                    function("g", seq(&[])),
                ],
            ),
        );
        let b = write(
            d.path(),
            "b.json",
            &birthmark(
                "b",
                BirthmarkType::OpSeq,
                vec![function("h", seq(&["RETURN"]))],
            ),
        );
        let report = compute(&[a, b], None);
        assert_eq!(report.groups.len(), 1);
        let g = &report.groups[0];
        assert_eq!(
            (g.ir.as_str(), g.birthmark_type.as_str()),
            ("ghidra-pcode", "op-seq")
        );
        assert_eq!(g.files, 2);
        // two functions in a, one in b
        let functions = g.functions.as_ref().unwrap();
        assert_eq!((functions.min, functions.max), (1, 2));
        // three elements, none, and one
        let elements = g.elements.as_ref().unwrap();
        assert_eq!((elements.min, elements.median, elements.max), (0, 1.0, 3));
        assert_eq!(g.empty, 1);
        assert_eq!(g.empty_ratio, Some(1.0 / 3.0));
        assert_eq!(g.vocabulary, 3);
        assert_eq!(g.occurrences, None);
        assert!(g.top.is_none());

        assert_eq!(report.files.len(), 2);
        assert_eq!((report.files[0].functions, report.files[0].empty), (2, 1));
    }

    #[test]
    fn test_groups_split_on_type_and_what_is_not_a_birthmark_is_skipped() {
        let d = tempfile::tempdir().unwrap();
        let set = Data::Set([s("COPY")].into_iter().collect());
        let a = write(
            d.path(),
            "a.json",
            &birthmark("a", BirthmarkType::OpSet, vec![function("f", set)]),
        );
        let b = write(
            d.path(),
            "b.json",
            &birthmark(
                "b",
                BirthmarkType::OpSeq,
                vec![function("f", seq(&["COPY"]))],
            ),
        );
        let junk = d.path().join("junk.json");
        std::fs::write(&junk, "{\"functions\": []}").unwrap();

        let report = compute(&[a, b, junk.clone()], None);
        let types = report
            .groups
            .iter()
            .map(|g| g.birthmark_type.as_str())
            .collect::<Vec<_>>();
        assert_eq!(types, vec!["op-seq", "op-set"]);
        assert_eq!(report.files.len(), 2);
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].path, junk);
        assert!(
            report.skipped[0].reason.contains("metadata"),
            "{}",
            report.skipped[0].reason
        );
    }

    #[test]
    fn test_freq_counts_occurrences_and_kgrams_are_one_element_each() {
        let d = tempfile::tempdir().unwrap();
        let k = |ops: &[&str]| Kgram::new(ops.iter().map(|o| s(o)).collect());
        let freq = Data::KgramFreq(
            [(k(&["COPY", "CALL"]), 3), (k(&["CALL", "RETURN"]), 1)]
                .into_iter()
                .collect(),
        );
        let a = write(
            d.path(),
            "a.json",
            &birthmark(
                "a",
                BirthmarkType::OpKgramFreq(2),
                vec![function("f", freq)],
            ),
        );
        let report = compute(&[a], Some(5));
        let g = &report.groups[0];
        // distinct keys, not their total
        assert_eq!(g.elements.as_ref().unwrap().max, 2);
        assert_eq!(g.occurrences, Some(4));
        assert_eq!(g.vocabulary, 2);
        assert_eq!(
            g.top.as_deref().unwrap(),
            [
                TopElement {
                    element: s("COPY CALL"),
                    count: 3
                },
                TopElement {
                    element: s("CALL RETURN"),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn test_top_breaks_ties_by_name_and_stops_at_n() {
        let d = tempfile::tempdir().unwrap();
        let a = write(
            d.path(),
            "a.json",
            &birthmark(
                "a",
                BirthmarkType::OpSeq,
                vec![function("f", seq(&["C", "B", "A", "A"]))],
            ),
        );
        let report = compute(&[a], Some(2));
        let top = report.groups[0].top.as_deref().unwrap();
        assert_eq!(
            top,
            [
                TopElement {
                    element: s("A"),
                    count: 2
                },
                TopElement {
                    element: s("B"),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn test_markdown_escapes_a_pipe_in_a_name() {
        let d = tempfile::tempdir().unwrap();
        let set = Data::Set([s("operator|")].into_iter().collect());
        let a = write(
            d.path(),
            "a.json",
            &birthmark("a", BirthmarkType::FcSet, vec![function("f", set)]),
        );
        let md = to_markdown(&compute(&[a], Some(1)), false);
        assert!(md.contains("`operator\\|`"), "{md}");
    }

    #[test]
    fn test_csv_refuses_per_file_and_top_together() {
        let d = tempfile::tempdir().unwrap();
        let opts = crate::cli::OinkieOpts::try_parse_from([
            "oinkie",
            "stats",
            "-f",
            "csv",
            "--per-file",
            "--top",
            "3",
            d.path().to_str().unwrap(),
        ])
        .unwrap();
        let crate::cli::OinkieCommand::Stats(opts) = opts.command else {
            panic!("Expected Stats command");
        };
        let e = perform(opts).unwrap_err();
        assert!(e.to_string().contains("one table"), "{e}");
    }

    #[test]
    fn test_output_goes_to_the_file_given() {
        let d = tempfile::tempdir().unwrap();
        write(
            d.path(),
            "a.json",
            &birthmark(
                "a",
                BirthmarkType::OpSeq,
                vec![function("f", seq(&["COPY"]))],
            ),
        );
        let out = d.path().join("stats.csv");
        let opts = crate::cli::OinkieOpts::try_parse_from([
            "oinkie",
            "stats",
            "-f",
            "csv",
            "-o",
            out.to_str().unwrap(),
            d.path().to_str().unwrap(),
        ])
        .unwrap();
        let crate::cli::OinkieCommand::Stats(opts) = opts.command else {
            panic!("Expected Stats command");
        };
        perform(opts).unwrap();
        let csv = std::fs::read_to_string(out).unwrap();
        let mut lines = csv.lines();
        assert!(
            lines
                .next()
                .unwrap()
                .starts_with("ir,birthmark_type,files,")
        );
        assert!(lines.next().unwrap().starts_with("ghidra-pcode,op-seq,1,"));
        assert_eq!(lines.next(), None);
    }
}
