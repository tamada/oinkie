//! What the command line offers a person, and what it says about each thing.
//!
//! The library knows what exists: `Ir::ALL`, `Algorithm::ALL`,
//! `PairingStrategy::ALL` and `BirthmarkType::all`. What to *suggest*, how to
//! spell it on a command line and what to say beside it is this module's.
//! clap's `ValueEnum` used to do that from inside the library, by deriving a
//! name and a help line from each variant's doc comment; the tables below are
//! those names and help lines, written where they are used.
//!
//! Each table holds the value itself next to its name, so a name cannot drift
//! from the thing it names. What is *not* guaranteed by construction is that
//! the table lists every variant, because nothing derives it any more; the
//! tests hold each one to the library's `ALL`.

use clap::builder::{PossibleValue, PossibleValuesParser, TypedValueParser};
use oinkie::birthmarks::BirthmarkType;
use oinkie::compare::{Algorithm, PairingStrategy};
use oinkie::lift::Ir;

/// How far the completion and help lists go for k-grams.
///
/// The parser has no ceiling -- `op-9gram-set-dice` is a valid name and always
/// was. This bounds only what gets *suggested*, because a list of completions
/// has to be finite, and it is set where `extract --birthmark-type` already
/// stopped.
pub const MAX_ADVERTISED_K: usize = 8;

type Table<T> = &'static [(T, &'static str, &'static str)];

/// Each representation, the name `--ir` takes, and its one-line description.
///
/// The IDA entry carries the cloud disclosure in its first sentence because
/// that is what `--help` and the shell completions show beside the name: a
/// reader choosing a representation should see it before choosing.
pub const IRS: Table<Ir> = &[
    (
        Ir::GhidraPcode,
        "ghidra-pcode",
        "Ghidra's P-Code, as refined by the decompiler — what `HighFunction` yields, rather than raw lifted P-Code",
    ),
    (
        Ir::IdaMicrocode,
        "ida-microcode",
        "The Hex-Rays microcode -- which runs IDA Pro's decompiler, so on an installation with a cloud decompiler each function is sent to Hex-Rays' servers",
    ),
    (
        Ir::BinaryNinjaLlil,
        "binary-ninja-llil",
        "Binary Ninja's Low Level IL: one expression per machine instruction, registers and flags still explicit",
    ),
    (
        Ir::BinaryNinjaMlil,
        "binary-ninja-mlil",
        "Binary Ninja's Medium Level IL: stack and registers resolved into variables, calls carrying their parameters",
    ),
    (
        Ir::BinaryNinjaHlil,
        "binary-ninja-hlil",
        "Binary Ninja's High Level IL: control flow recovered, the level its decompiler output is rendered from",
    ),
];

/// Each algorithm, the name `--algorithm` takes, and its one-line description.
pub const ALGORITHMS: Table<Algorithm> = &[
    (
        Algorithm::Cosine,
        "cosine",
        "Cosine similarity based on term frequency vectors. Available: seq and freq",
    ),
    (
        Algorithm::Dice,
        "dice",
        "Dice coefficient. Available: seq, set and freq",
    ),
    (
        Algorithm::Euclidean,
        "euclidean",
        "Euclidean distance between term frequency vectors. Available: seq and freq",
    ),
    (
        Algorithm::Jaccard,
        "jaccard",
        "Jaccard index. Available: seq, set and freq",
    ),
    (
        Algorithm::Levenshtein,
        "levenshtein",
        "Levenshtein distance. Available: seq",
    ),
    (
        Algorithm::Lcs,
        "lcs",
        "Longest Common Subsequence (LCS). Available: seq",
    ),
    (
        Algorithm::Simpson,
        "simpson",
        "Simpson's coefficient. Available: seq, set and freq",
    ),
    (
        Algorithm::WeightedJaccard,
        "weighted-jaccard",
        "Weighted Jaccard index based on term frequency vectors. Available: seq and freq",
    ),
];

/// Each pairing strategy, the name `--strategy` takes, and its description.
pub const STRATEGIES: Table<PairingStrategy> = &[
    (
        PairingStrategy::AllAndSelf,
        "all-and-self",
        "All possible combinations including self-comparisons ($_nC_2 + n$). Used for full matrix visualization or comprehensive heatmaps",
    ),
    (
        PairingStrategy::All,
        "all",
        "Compares all possible combinations ($_nC_2$). Used for comprehensive validation of accuracy (False Positive / True Positive)",
    ),
    (
        PairingStrategy::SelfCoverage,
        "self-coverage",
        "Compares each file with itself ($n$). Used for sanity checks to ensure identical files yield a similarity score of 1.0",
    ),
    (
        PairingStrategy::Adjacent,
        "adjacent",
        "Compares only adjacent pairs in the list ($n-1$). Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2)",
    ),
    (
        PairingStrategy::FirstVsOthers,
        "first-vs-others",
        "Compares a specific reference file against all other files ($n-1$). Compares first item and all other items. Useful for comparing a baseline version against multiple variants",
    ),
    (
        PairingStrategy::LastVsOthers,
        "last-vs-others",
        "Compares a specific reference file against all other files ($n-1$). Compares the last item and all other items. Useful for comparing a baseline version against multiple variants",
    ),
];

/// The value `name` stands for, in any case, or clap's own wording of why not.
///
/// The wording is `ValueEnum::from_str`'s, which is what the MCP server's
/// callers have always been shown.
pub fn by_name<T: Clone>(table: Table<T>, name: &str) -> Result<T, String> {
    let wanted = name.to_lowercase();
    table
        .iter()
        .find(|(_, n, _)| *n == wanted)
        .map(|(v, _, _)| v.clone())
        .ok_or_else(|| format!("invalid variant: {name}"))
}

/// A value parser that offers a table's names, with their descriptions, and
/// yields the value each one stands for.
///
/// Built on clap's own `PossibleValuesParser`, so the help, the completions,
/// the "did you mean" suggestions and `ignore_case` behave as they did when
/// these were `ValueEnum`s.
fn table_parser<T: Clone + Send + Sync + 'static>(
    table: Table<T>,
) -> impl TypedValueParser<Value = T> {
    PossibleValuesParser::new(
        table
            .iter()
            .map(|(_, name, help)| PossibleValue::new(*name).help(*help)),
    )
    .map(move |name| by_name(table, &name).expect("the parser yields only names in the table"))
}

pub fn ir_parser() -> impl TypedValueParser<Value = Ir> {
    table_parser(IRS)
}

pub fn algorithm_parser() -> impl TypedValueParser<Value = Algorithm> {
    table_parser(ALGORITHMS)
}

pub fn strategy_parser() -> impl TypedValueParser<Value = PairingStrategy> {
    table_parser(STRATEGIES)
}

/// The birthmark names offered: every family, with k-grams up to
/// [`MAX_ADVERTISED_K`].
///
/// What completion and `--help` list, not what the parser accepts --
/// `BirthmarkType::try_from` takes any k.
pub fn advertised_birthmarks() -> Vec<BirthmarkType> {
    BirthmarkType::all(MAX_ADVERTISED_K)
}

/// Every canonical `{birthmark}-{algorithm}` name offered.
///
/// Generated by pairing each advertised birthmark with the algorithms that
/// operate on its shape, so the list cannot offer a pairing the library would
/// refuse -- which a hand-written list could, and which is what made the
/// copy this replaced worth deleting (#25).
///
/// This is not the set of names that parse: any k does, and this stops at
/// [`MAX_ADVERTISED_K`].
pub fn advertised_analyses() -> Vec<String> {
    let mut names = Vec::new();
    for birthmark in advertised_birthmarks() {
        for algorithm in Algorithm::ALL {
            if birthmark.pairs_with(algorithm) {
                names.push(format!("{birthmark}-{}", algorithm.name()));
            }
        }
    }
    names
}

/// One line saying what a birthmark holds, for `oinkie info` and for the help
/// text of `--birthmark-type`.
///
/// Phrased without a trailing full stop, as clap renders a short help.
pub fn describe(birthmark: &BirthmarkType) -> String {
    own_description(birthmark).unwrap_or_else(|| format!("the {birthmark} birthmark of a program"))
}

/// The description written for a birthmark family, or `None` for one nobody
/// has written yet.
///
/// `BirthmarkType` is `#[non_exhaustive]`, so this `match` needs a wildcard
/// arm and the compiler no longer stops at a family added to the library.
/// A test does instead: every family `BirthmarkType::all` lists must have its
/// own description here, and the library holds `all` to the enum.
fn own_description(birthmark: &BirthmarkType) -> Option<String> {
    Some(match birthmark {
        BirthmarkType::FcSeq => "the sequence of method calls in a program".to_string(),
        BirthmarkType::FcFreq => "the frequency of method calls in a program".to_string(),
        BirthmarkType::FcSet => "the set of method calls in a program".to_string(),
        BirthmarkType::OpSeq => "the sequence of operations in a program".to_string(),
        BirthmarkType::OpSet => "the set of operations in a program".to_string(),
        BirthmarkType::OpFreq => "the frequency of operations in a program".to_string(),
        BirthmarkType::OpKgramSeq(k) => {
            format!("the sequence of {k}-grams of operations in a program")
        }
        BirthmarkType::OpKgramFreq(k) => {
            format!("the frequency of {k}-grams of operations in a program")
        }
        BirthmarkType::OpKgramSet(k) => {
            format!("the set of {k}-grams of operations in a program")
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oinkie::birthmarks::AnalysisType;

    /// A table is written by hand, so it is held to the library's list: the
    /// same values, each once. Without this a variant added to the library
    /// would be accepted by every API and absent from every command line.
    fn assert_covers<T: PartialEq + std::fmt::Debug>(table: Table<T>, all: &[T]) {
        assert_eq!(table.len(), all.len(), "the table and ALL differ in length");
        for v in all {
            let listed = table.iter().filter(|(t, _, _)| t == v).count();
            assert_eq!(listed, 1, "{v:?} is listed {listed} times");
        }
        let mut names = table.iter().map(|(_, n, _)| *n).collect::<Vec<_>>();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), table.len(), "two entries share a name");
        for (v, name, help) in table {
            assert!(!help.is_empty(), "{v:?} ({name}) has no description");
        }
    }

    #[test]
    fn test_every_table_lists_what_the_library_lists() {
        assert_covers(IRS, Ir::ALL);
        assert_covers(ALGORITHMS, Algorithm::ALL);
        assert_covers(STRATEGIES, PairingStrategy::ALL);
    }

    /// The name `--ir` takes is the name the library writes into a file, so
    /// the two cannot be allowed to differ.
    #[test]
    fn test_an_ir_is_offered_under_the_name_the_library_gives_it() {
        for (ir, name, _) in IRS {
            assert_eq!(*name, ir.to_string());
        }
    }

    /// An algorithm's offered name has to be one the library reads back, since
    /// it is what `oinkie info` prints and people build analysis names from it.
    #[test]
    fn test_an_algorithm_is_offered_under_a_name_the_library_reads() {
        for (algorithm, name, _) in ALGORITHMS {
            assert_eq!(&name.parse::<Algorithm>().unwrap(), algorithm, "{name}");
        }
    }

    /// The defaults written into the option definitions are strings, so they
    /// are checked against the values they are meant to be.
    #[test]
    fn test_the_defaults_name_what_they_mean() {
        assert_eq!(by_name(IRS, "ghidra-pcode"), Ok(Ir::default()));
        assert_eq!(by_name(ALGORITHMS, "jaccard"), Ok(Algorithm::Jaccard));
        assert_eq!(
            by_name(STRATEGIES, "all-and-self"),
            Ok(PairingStrategy::AllAndSelf)
        );
    }

    #[test]
    fn test_a_name_is_read_in_any_case_and_refused_otherwise() {
        assert_eq!(by_name(ALGORITHMS, "LCS"), Ok(Algorithm::Lcs));
        assert_eq!(
            by_name(ALGORITHMS, "nonsense"),
            Err("invalid variant: nonsense".to_string())
        );
    }

    #[test]
    fn test_the_ida_entry_says_where_the_functions_may_go() {
        let (_, _, help) = IRS
            .iter()
            .find(|(ir, _, _)| *ir == Ir::IdaMicrocode)
            .unwrap();
        assert!(help.contains("sent to Hex-Rays"), "{help}");
    }

    /// Every name offered has to parse back, and the list stops at the ceiling
    /// while the parser does not.
    #[test]
    fn test_every_advertised_analysis_name_parses() {
        let names = advertised_analyses();
        for name in &names {
            AnalysisType::try_from(name.as_str()).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        // Two families that have no k, plus MAX_ADVERTISED_K that do, each in
        // three shapes; and each shape pairs with the algorithms that operate
        // on it -- two for seq, three each for set and freq.
        assert_eq!(names.len(), (2 + 3 + 3) * (2 + MAX_ADVERTISED_K));
    }

    #[test]
    fn test_the_advertised_ceiling_does_not_bound_the_parser() {
        let name = format!("op-{}gram-set-dice", MAX_ADVERTISED_K + 1);
        assert!(
            !advertised_analyses().contains(&name),
            "{name} should be past the end of the list"
        );
        assert!(AnalysisType::try_from(name.as_str()).is_ok(), "{name}");
    }

    /// A generated list would offer pairings that the library refuses if it
    /// crossed every algorithm with every birthmark. It filters by shape, and
    /// this is what says it filters rather than merely happening to come out
    /// valid.
    #[test]
    fn test_the_advertised_names_omit_the_pairings_that_are_refused() {
        let offered = advertised_analyses();
        for name in ["op-seq-euclidean", "op-set-levenshtein", "op-3gram-set-lcs"] {
            assert!(
                AnalysisType::try_from(name).is_err(),
                "{name} should be refused"
            );
            assert!(
                !offered.iter().any(|n| n == name),
                "{name} is refused but still advertised"
            );
        }
    }

    /// The wildcard arm in `own_description` is for families nobody has
    /// described yet, and this says there are none: every family the library
    /// lists has words of its own, not the generic fallback.
    #[test]
    fn test_every_birthmark_family_has_its_own_description() {
        for bt in BirthmarkType::all(MAX_ADVERTISED_K) {
            assert!(
                own_description(&bt).is_some(),
                "{bt} is described only by the fallback; write it a description"
            );
        }
    }

    #[test]
    fn test_every_advertised_birthmark_describes_itself() {
        for bt in advertised_birthmarks() {
            let name = bt.to_string();
            let described = describe(&bt);
            assert!(!described.is_empty(), "{name} has no description");
            assert!(
                !described.ends_with('.'),
                "{name}: clap renders a short help without a full stop: {described}"
            );
            if let BirthmarkType::OpKgramSeq(k)
            | BirthmarkType::OpKgramFreq(k)
            | BirthmarkType::OpKgramSet(k) = &bt
            {
                assert!(
                    described.contains(&format!("{k}-gram")),
                    "{name} does not say which k: {described}"
                );
            }
        }
    }

    /// The property the issue was really about: every name a person can build
    /// out of what `oinkie info` shows them has to parse.
    ///
    /// Generated from the two lists rather than written out, so a two-word
    /// algorithm added later is covered without anyone remembering to.
    #[test]
    fn test_a_name_built_from_what_info_prints_parses() {
        for birthmark in advertised_birthmarks() {
            for (algorithm, shown, _) in ALGORITHMS {
                if !birthmark.pairs_with(algorithm) {
                    continue;
                }
                let name = format!("{birthmark}-{shown}");
                let parsed =
                    AnalysisType::try_from(name.as_str()).unwrap_or_else(|e| panic!("{name}: {e}"));
                assert_eq!(parsed.birthmark(), &birthmark, "{name}");
            }
        }
    }
}
