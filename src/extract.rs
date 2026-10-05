use rustc_hash::FxHashMap;

use crate::birthmarks::{Birthmark, BirthmarkType, Data, Function, Kgram, Metadata};
use crate::program::{TypedFunction, TypedProgram};
use crate::{Iterable, Result};

pub struct Extractor {
    bt: BirthmarkType,
}

impl Extractor {
    pub fn new(bt: BirthmarkType) -> Self {
        Self { bt }
    }

    /// The birthmark this extractor makes.
    pub fn birthmark_type(&self) -> &BirthmarkType {
        &self.bt
    }

    pub(crate) fn extract_each_typed<T: crate::Op>(
        &self,
        p: &TypedProgram<T>,
    ) -> Result<Birthmark> {
        extract_birthmark_op(p, &self.bt)
    }

    /// Extracts from a program read without naming its operation type.
    ///
    /// The birthmark is the same either way; this only spares the caller from
    /// deciding which lifter produced the file, which the file already says.
    pub fn extract(&self, p: &crate::program::Program) -> Result<Birthmark> {
        use crate::program::Lifted;
        match &p.0 {
            Lifted::GhidraPcode(p) => self.extract_each_typed(p),
            Lifted::BinaryNinjaLlil(p) => self.extract_each_typed(p),
            Lifted::BinaryNinjaMlil(p) => self.extract_each_typed(p),
            Lifted::BinaryNinjaHlil(p) => self.extract_each_typed(p),
            Lifted::IdaMicrocode(p) => self.extract_each_typed(p),
        }
    }
}

fn extract_birthmark_op<T: crate::Op>(
    p: &TypedProgram<T>,
    bt: &BirthmarkType,
) -> Result<Birthmark> {
    // A name with k = 0 does not parse, but the variant can be built by hand,
    // and a window of size zero panics. Refused as the unknown birthmark it is.
    if let BirthmarkType::OpKgramSeq(0)
    | BirthmarkType::OpKgramSet(0)
    | BirthmarkType::OpKgramFreq(0) = bt
    {
        return Err(crate::Error::BirthmarkType(bt.to_string()));
    }
    let now = std::time::Instant::now();
    // An empty fc-* birthmark is a measurement, not a failure: a program that
    // calls nothing is a program that calls nothing, and the family simply does
    // not distinguish it. Said rather than refused, because the two empty
    // birthmarks it produces will score 1.0 against each other and the reader
    // should know which of those two things they are looking at.
    if matches!(
        bt,
        BirthmarkType::FcSeq | BirthmarkType::FcSet | BirthmarkType::FcFreq
    ) && let Some(reason) = empty_family(p)
    {
        log::warn!("{}", reason.message(p.path(), p.ir()));
    }
    let functions = p
        .iter()
        .map(|f| {
            let name = f.name().to_string();
            let data = match bt {
                BirthmarkType::FcFreq => Data::Freq(extract_function_calls_freq(f, p)),
                BirthmarkType::FcSet => {
                    Data::Set(extract_function_calls_freq(f, p).into_keys().collect())
                }
                BirthmarkType::FcSeq => Data::Seq(extract_function_calls(f, p)),
                BirthmarkType::OpFreq => Data::Freq(f.ops_freq()),
                BirthmarkType::OpSet => Data::Set(f.ops_freq().into_keys().collect()),
                BirthmarkType::OpSeq => Data::Seq(f.ops().map(|s| s.into()).collect()),
                BirthmarkType::OpKgramSeq(k) => Data::KgramSeq(extract_op_kgram_seq(f, *k)),
                BirthmarkType::OpKgramFreq(k) => Data::KgramFreq(extract_op_kgram_freq(f, *k)),
                BirthmarkType::OpKgramSet(k) => {
                    Data::KgramSet(extract_op_kgram_seq(f, *k).into_iter().collect())
                }
            };
            Function { name, data }
        })
        .collect::<Vec<_>>();
    let metadata = build_metadata(p, bt.clone(), now);
    Ok(Birthmark {
        metadata,
        functions,
        json_path: None,
    })
}

fn build_metadata<T>(
    p: &TypedProgram<T>,
    bt: BirthmarkType,
    start_time: std::time::Instant,
) -> Metadata {
    let extracted_at = chrono::Utc::now();
    let duration = start_time.elapsed();
    let path = p.path().to_path_buf();
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    Metadata {
        file_name: name,
        path,
        extracted_at,
        duration,
        birthmark_type: bt,
        ir: p.ir(),
    }
}

fn extract_op_kgram_seq<T: crate::Op>(f: &TypedFunction<T>, k: usize) -> Vec<Kgram> {
    f.ops()
        .map(|s| s.into())
        .collect::<Vec<_>>()
        .windows(k)
        .map(|w| Kgram::new(w.to_vec()))
        .collect()
}

fn extract_op_kgram_freq<T: crate::Op>(f: &TypedFunction<T>, k: usize) -> FxHashMap<Kgram, usize> {
    seq_to_freq(extract_op_kgram_seq(f, k).into_iter())
}

pub(crate) fn seq_to_freq<T>(seq: impl Iterator<Item = T>) -> FxHashMap<T, usize>
where
    T: std::hash::Hash + Eq,
{
    seq.into_iter().fold(FxHashMap::default(), |mut acc, item| {
        *acc.entry(item).or_insert(0) += 1;
        acc
    })
}

/// Why every `fc-*` birthmark of a program would be empty, or `None` when one
/// of them would not be.
///
/// Returned rather than logged so that the decision can be tested without a
/// logger, and so that the caller owns how loudly it is said.
///
/// The two cases are kept apart because they send the reader somewhere else.
/// Nothing being a call is `is_call`; calls that resolve to nothing is the
/// symbol table or `symbol_key`. Both were reached by a real lifter during
/// v0.6.0 -- the Hex-Rays microcode writes a global as `$name` and the table
/// was keyed by the resolved name, and the optimiser folds a call into another
/// instruction's operand, which a reader walking only the block's list misses.
#[derive(Debug, PartialEq, Eq)]
enum EmptyFamily {
    /// No operation in the program is a call.
    NoCalls,
    /// There are calls, and none of them names anything the symbol table holds.
    NoneResolve(usize),
}

impl EmptyFamily {
    fn message(&self, path: &std::path::Path, ir: crate::lift::Ir) -> String {
        let path = path.display();
        match self {
            Self::NoCalls => format!(
                "{path}: no operation is a call, so every fc-* birthmark of it is empty -- and \
                 two empty birthmarks score as a perfect match. Either the program really calls \
                 nothing, or oinkie's reader for {ir} does not recognise that representation's \
                 call operations"
            ),
            Self::NoneResolve(calls) => format!(
                "{path}: calls were found ({calls} in total), but none of them names anything in \
                 the symbol table, so every fc-* birthmark of it is empty -- and two empty \
                 birthmarks score as a perfect match. Either every call is indirect, or oinkie's \
                 reader for {ir} keys the symbol table differently from the way that \
                 representation renders a callee"
            ),
        }
    }
}

/// Decides which, if either, applies.
///
/// One pass, returning at the first call that resolves. The count is only read
/// when none did, and reaching that answer means the whole program was walked
/// anyway -- so counting here costs an extraction that has calls nothing, where
/// counting first cost it a full traversal for a number it threw away.
fn empty_family<T: crate::Op>(p: &TypedProgram<T>) -> Option<EmptyFamily> {
    let mut calls = 0usize;
    for function in p.iter() {
        for op in function.iter() {
            if !op.is_call() {
                continue;
            }
            calls += 1;
            if op.symbol_key().and_then(|key| p.symbol(&key)).is_some() {
                return None;
            }
        }
    }
    Some(if calls == 0 {
        EmptyFamily::NoCalls
    } else {
        EmptyFamily::NoneResolve(calls)
    })
}

fn extract_function_calls<T: crate::Op>(f: &TypedFunction<T>, p: &TypedProgram<T>) -> Vec<String> {
    f.iter()
        .filter(|op| op.is_call())
        .filter_map(|op| op.symbol_key())
        .filter_map(|key| p.symbol(&key))
        .map(|s| s.to_string())
        .collect()
}

fn extract_function_calls_freq<T: crate::Op>(
    f: &TypedFunction<T>,
    p: &TypedProgram<T>,
) -> FxHashMap<String, usize> {
    extract_function_calls(f, p)
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, call| {
            *acc.entry(call).or_insert(0) += 1;
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A zero k-gram cannot be parsed, but the variant can be built by hand,
    /// and extracting it is refused rather than panicking on a window of size
    /// zero.
    #[test]
    fn test_a_zero_gram_is_refused_rather_than_panicking() {
        let p =
            crate::Program::load(std::path::Path::new("testdata/lifted/pcodes/udl.json")).unwrap();
        for bt in [
            BirthmarkType::OpKgramSeq(0),
            BirthmarkType::OpKgramSet(0),
            BirthmarkType::OpKgramFreq(0),
        ] {
            let err = Extractor::new(bt.clone())
                .extract(&p)
                .err()
                .unwrap_or_else(|| panic!("{bt} was extracted"));
            assert!(err.to_string().contains(&bt.to_string()), "{err}");
        }
    }
    use tempfile::tempdir;

    /// A program whose calls resolve to nothing, written by hand rather than
    /// lifted: the reader does not care where the JSON came from, and no real
    /// binary is needed to describe a call through a register.
    fn a_program_whose_calls_resolve_to_nothing() -> crate::program::Program {
        let dir = tempdir().unwrap();
        let path = dir.path().join("indirect.json");
        // CALLIND is a call, and `symbol_key` yields nothing for one: its
        // target lives in a register, resolved at run time.
        std::fs::write(
            &path,
            r#"{"program":"indirect","path":"bin/indirect","ir":"ghidra-pcode",
                "symbols":{"0x1000": "_printf"},
                "functions":[{"name":"main","ops":[
                  {"op":"CALLIND","inputs":["(register, 0x20, 8)"]},
                  {"op":"CALLIND","inputs":["(register, 0x28, 8)"]}]}]}"#,
        )
        .unwrap();
        crate::program::Program::load(&path).unwrap()
    }

    /// An empty family is returned rather than refused. A program that calls
    /// nothing is a program that calls nothing, and `fc-*` simply does not
    /// distinguish it -- which is a property of the measure, not a failure.
    #[test]
    fn test_calls_that_resolve_to_nothing_give_an_empty_birthmark() {
        let p = a_program_whose_calls_resolve_to_nothing();
        for bt in ["fc-set", "fc-seq", "fc-freq"] {
            let b = Extractor::new(BirthmarkType::try_from(bt).unwrap())
                .extract(&p)
                .unwrap_or_else(|e| panic!("{bt} was refused: {e}"));
            assert_eq!(b.len(), 1, "{bt}: one function was expected");
            let calls: Vec<String> = b.iter().flat_map(|e| e.ops().map(String::from)).collect();
            assert!(calls.is_empty(), "{bt}: expected nothing, got {calls:?}");
        }
    }

    /// One program that cannot answer does not stop the others.
    ///
    /// It used to: the refusal was an `Err`, and a corpus with a single
    /// call-less binary in it lost every birthmark after that one.
    #[test]
    fn test_a_program_that_calls_nothing_does_not_stop_the_rest() {
        let dir = tempdir().unwrap();
        let mut programs = Vec::new();
        for (name, ops) in [
            ("calls", r#"{"op":"CALL","inputs":["(ram, 0x1000, 8)"]}"#),
            (
                "pure",
                r#"{"op":"COPY","out":"(register, 0x0, 8)","inputs":["(const, 0x1, 8)"]}"#,
            ),
            (
                "calls_too",
                r#"{"op":"CALL","inputs":["(ram, 0x1000, 8)"]}"#,
            ),
        ] {
            let path = dir.path().join(format!("{name}.json"));
            std::fs::write(
                &path,
                format!(
                    r#"{{"program":"{name}","path":"bin/{name}","ir":"ghidra-pcode",
                        "symbols":{{"0x1000":"_printf"}},
                        "functions":[{{"name":"main","ops":[{ops}]}}]}}"#
                ),
            )
            .unwrap();
            programs.push(crate::program::Program::load(&path).unwrap());
        }
        let extractor = Extractor::new(BirthmarkType::try_from("fc-set").unwrap());
        let sizes: Vec<usize> = programs
            .iter()
            .map(|p| {
                extractor
                    .extract(p)
                    .unwrap_or_else(|e| panic!("{e}"))
                    .iter()
                    .flat_map(|e| e.ops())
                    .count()
            })
            .collect();
        assert_eq!(
            sizes,
            vec![1, 0, 1],
            "the middle one should be empty, not fatal"
        );
    }

    /// Which of the two emptied the family, and that neither fires when one
    /// call resolves.
    ///
    /// This is the part worth keeping from the old refusal: the two send the
    /// reader somewhere else, and both were reached by a real lifter during
    /// v0.6.0.
    #[test]
    fn test_the_two_ways_of_emptying_a_family_are_told_apart() {
        let unresolved = a_program_whose_calls_resolve_to_nothing();
        assert!(matches!(
            any_empty_family(&unresolved),
            Some(EmptyFamily::NoneResolve(2))
        ));

        let dir = tempdir().unwrap();
        let path = dir.path().join("callless.json");
        std::fs::write(
            &path,
            r#"{"program":"callless","path":"bin/callless","ir":"ghidra-pcode",
                "symbols":{},
                "functions":[{"name":"main","ops":[
                  {"op":"COPY","out":"(register, 0x0, 8)","inputs":["(const, 0x1, 8)"]}]}]}"#,
        )
        .unwrap();
        let callless = crate::program::Program::load(&path).unwrap();
        assert_eq!(any_empty_family(&callless), Some(EmptyFamily::NoCalls));

        let fine = crate::program::Program::load(std::path::Path::new(
            "testdata/lifted/pcodes/hello_clang.json",
        ))
        .unwrap();
        assert_eq!(any_empty_family(&fine), None);
    }

    /// The messages say which case they are, and name the file and the
    /// representation -- the two things that tell a lifter bug from a program
    /// that really calls nothing.
    #[test]
    fn test_each_message_names_the_file_the_representation_and_the_cause() {
        let path = std::path::Path::new("bin/sample");
        let ir = crate::lift::Ir::GhidraPcode;

        let none = EmptyFamily::NoCalls.message(path, ir);
        assert!(
            none.contains("bin/sample") && none.contains("ghidra-pcode"),
            "{none}"
        );
        assert!(none.contains("no operation is a call"), "{none}");

        let some = EmptyFamily::NoneResolve(1).message(path, ir);
        assert!(some.contains("symbol table"), "{some}");
        assert!(
            !some.contains("1 operations"),
            "reads as a plural for one: {some}"
        );
        assert!(some.contains("(1 in total)"), "{some}");
    }

    /// `empty_family` is generic over the operation type, and the tests hold an
    /// `Program`. This is the one place that has to know which it is.
    fn any_empty_family(p: &crate::program::Program) -> Option<EmptyFamily> {
        match &p.0 {
            crate::program::Lifted::GhidraPcode(p) => empty_family(p),
            _ => unreachable!("the fixtures here are all P-Code"),
        }
    }

    /// The op-* families say nothing about calls, so neither warning applies
    /// to them. A program of unresolvable calls still has operations.
    #[test]
    fn test_the_op_families_are_not_refused_for_unresolvable_calls() {
        let p = a_program_whose_calls_resolve_to_nothing();
        let b = Extractor::new(BirthmarkType::try_from("op-set").unwrap())
            .extract(&p)
            .expect("op-set does not depend on symbols");
        assert_eq!(b.len(), 1);
    }

    /// The fc-* family is built from the symbols a function calls. The call
    /// operand names its target in the lifter's own notation, which is not the
    /// form the symbol table is keyed by, so the two have to be reconciled
    /// before the lookup — otherwise every call is discarded and the birthmark
    /// comes out empty.
    #[test]
    fn test_extract_function_calls_resolves_the_symbol() {
        let program: TypedProgram<crate::ghidra::Op> =
            std::path::Path::new("testdata/lifted/pcodes/hello_clang.json")
                .try_into()
                .expect("failed to load the fixture");
        let function = program.iter().next().expect("fixture has no function");

        let calls = extract_function_calls(function, &program);
        assert_eq!(
            calls,
            vec!["_printf".to_string()],
            "the fixture calls _printf exactly once"
        );
    }

    /// Two empty sets are identical, so an fc-* birthmark that resolves nothing
    /// makes unrelated programs look like a perfect match. Guard the property
    /// that makes the emptiness dangerous, not just the emptiness.
    #[test]
    fn test_fc_birthmarks_are_not_empty() {
        for fixture in [
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/hello_gcc.json",
        ] {
            let program: TypedProgram<crate::ghidra::Op> = std::path::Path::new(fixture)
                .try_into()
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            let birthmark = Extractor::new(BirthmarkType::FcSet)
                .extract_each_typed(&program)
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert!(
                birthmark.functions.iter().any(|e| !e.is_empty()),
                "{fixture}: every fc-set element is empty"
            );
        }
    }

    /// A leaf function calls nothing, so its fc-* birthmark is empty. That is
    /// the measurement, and it is handed back.
    ///
    /// It used to be refused. Two empty birthmarks score as a perfect match,
    /// and in a theft-detection tool that reads as a positive -- but the answer
    /// to that is to say so, not to decide on the caller's behalf that the
    /// question cannot be asked. `empty_family` is what says so.
    #[test]
    fn test_fc_extraction_of_a_program_without_calls_is_empty_not_refused() {
        let json = r#"{
            "program": "callless",
            "path": "bin/callless",
            "ir": "ghidra-pcode",
            "symbols": {"0x100000480": "_printf"},
            "functions": [
                {"name": "leaf", "ops": [
                    {"op": "COPY", "out": "(register, 0x4000, 8)", "inputs": ["(const, 0x0, 8)"]},
                    {"op": "RETURN", "inputs": ["(const, 0x0, 8)"]}
                ]}
            ]
        }"#;
        let program: TypedProgram<crate::ghidra::Op> = serde_json::from_str(json).unwrap();

        for bt in [
            BirthmarkType::FcSeq,
            BirthmarkType::FcSet,
            BirthmarkType::FcFreq,
        ] {
            let b = Extractor::new(bt.clone())
                .extract_each_typed(&program)
                .unwrap_or_else(|e| panic!("{bt} was refused: {e}"));
            assert!(
                b.iter().flat_map(|e| e.ops()).next().is_none(),
                "{bt}: a leaf function calls nothing, so this should be empty"
            );
        }
    }

    /// The op-* families do not read calls at all, so the refusal above must
    /// not spread to them.
    #[test]
    fn test_op_extraction_accepts_a_program_without_calls() {
        let json = r#"{
            "program": "callless",
            "path": "bin/callless",
            "symbols": {},
            "functions": [
                {"name": "leaf", "ops": [
                    {"op": "RETURN", "inputs": ["(const, 0x0, 8)"]}
                ]}
            ]
        }"#;
        let program: TypedProgram<crate::ghidra::Op> = serde_json::from_str(json).unwrap();
        assert!(
            Extractor::new(BirthmarkType::OpSeq)
                .extract_each_typed(&program)
                .is_ok()
        );
    }
}
