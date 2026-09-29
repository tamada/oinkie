use std::path::Path;

use rustc_hash::FxHashMap;

use crate::birthmarks::{Birthmark, BirthmarkType, Data, Elements, Kgram, Metadata};
use crate::program::{Function, Program};
use crate::{Error, Iterable, Result};

/// Number of hex characters kept from the SHA-256 digest. 16 characters
/// is 64 bits, which puts the birthday bound around 5 billion files;
/// 8 characters (32 bits) collided at roughly 77,000.
const HASH_PREFIX_LEN: usize = 16;

/// Generates the file name for the extracted birthmark JSON file.
/// The format of resultant file name is `{original_file_stem}_{hash}.json`,
/// where `original_file_stem` is the stem of the input source file and
/// `hash` is a hash value generated from the content of the source file
/// to ensure uniqueness and avoid overwriting files with the same name.
pub fn dest_file_name(program_path: &Path) -> Result<String> {
    let file_name = program_path
        .file_stem()
        .ok_or_else(|| {
            Error::Parse(format!(
                "{}: cannot determine the file stem",
                program_path.display()
            ))
        })?
        .to_string_lossy();
    let hash = get_hash(program_path);
    let new_filename = format!("{file_name}_{}.json", hash?);
    Ok(new_filename)
}

fn get_hash(path: &Path) -> Result<String> {
    use sha2::Digest;
    use std::io::{Read, Seek};
    let pbuf = path.to_path_buf();

    let mut file = std::fs::File::open(path).map_err(|e| Error::Io(pbuf.clone(), e))?;
    let len = file
        .metadata()
        .map_err(|e| Error::Io(pbuf.clone(), e))?
        .len();

    let mut hasher = sha2::Sha256::new();
    // the file length participates in the hash so that same-prefix/suffix
    // files of different sizes never collide
    hasher.update(len.to_le_bytes());

    // Read the first 4KB of the file for hashing
    let mut head = vec![0; 4096.min(len as usize)];
    file.read_exact(&mut head)
        .map_err(|e| Error::Io(pbuf.clone(), e))?;
    hasher.update(&head);

    if len > 4096 {
        // Read the last (up to) 4KB of the file, without overlapping the head
        let tail_len = 4096.min(len as usize - 4096);
        let mut tail = vec![0; tail_len];
        file.seek(std::io::SeekFrom::End(-(tail_len as i64)))
            .map_err(|e| Error::Io(pbuf.clone(), e))?;
        file.read_exact(&mut tail)
            .map_err(|e| Error::Io(pbuf.clone(), e))?;
        hasher.update(&tail);
    }
    let hash = hasher.finalize();
    // Written out rather than `format!("{:x}")`: sha2 0.11 returns an
    // `Output` that does not implement `LowerHex`. Only the prefix is kept, so
    // only that many bytes are rendered.
    Ok(hash
        .iter()
        .take(HASH_PREFIX_LEN.div_ceil(2))
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..HASH_PREFIX_LEN]
        .to_string())
}

pub struct Extractor {
    bt: BirthmarkType,
}

impl Extractor {
    pub fn new(bt: BirthmarkType) -> Self {
        Self { bt }
    }

    pub fn extract<T: crate::Op>(&self, args: Vec<&Program<T>>) -> Result<Vec<Birthmark>> {
        let result = args
            .iter()
            .map(|p| extract_birthmark_op(p, &self.bt))
            .collect::<Vec<_>>();
        Error::vec_result_to_result_vec(result)
    }

    pub fn extract_each<T: crate::Op>(&self, p: &Program<T>) -> Result<Birthmark> {
        extract_birthmark_op(p, &self.bt)
    }

    /// Extracts from a program read without naming its operation type.
    ///
    /// The birthmark is the same either way; this only spares the caller from
    /// deciding which lifter produced the file, which the file already says.
    pub fn extract_any(&self, p: &crate::program::AnyProgram) -> Result<Birthmark> {
        use crate::program::AnyProgram;
        match p {
            AnyProgram::GhidraPcode(p) => self.extract_each(p),
            AnyProgram::BinaryNinjaLlil(p) => self.extract_each(p),
            AnyProgram::BinaryNinjaMlil(p) => self.extract_each(p),
            AnyProgram::BinaryNinjaHlil(p) => self.extract_each(p),
            AnyProgram::IdaMicrocode(p) => self.extract_each(p),
        }
    }
}

fn extract_birthmark_op<T: crate::Op>(p: &Program<T>, bt: &BirthmarkType) -> Result<Birthmark> {
    let now = std::time::Instant::now();
    // An fc-* birthmark of a program that calls nothing is empty, and two empty
    // birthmarks score 1.0 against each other, so unrelated programs come back
    // identical. Both ways of reaching that are refused here.
    if matches!(
        bt,
        BirthmarkType::FcSeq | BirthmarkType::FcSet | BirthmarkType::FcFreq
    ) {
        refuse_an_empty_family(p)?;
    }
    let elements = p
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
            Elements { name, data }
        })
        .collect::<Vec<_>>();
    let metadata = build_metadata(p, bt.clone(), now);
    Ok(Birthmark {
        metadata,
        elements,
        json_path: None,
    })
}

fn build_metadata<T>(
    p: &Program<T>,
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

fn extract_op_kgram_seq<T: crate::Op>(f: &Function<T>, k: usize) -> Vec<Kgram> {
    f.ops()
        .map(|s| s.into())
        .collect::<Vec<_>>()
        .windows(k)
        .map(|w| Kgram::new(w.to_vec()))
        .collect()
}

fn extract_op_kgram_freq<T: crate::Op>(f: &Function<T>, k: usize) -> FxHashMap<Kgram, usize> {
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

/// Refuses a program from which no `fc-*` birthmark could hold anything.
///
/// `extract_function_calls` is three filters -- is it a call, does it name a
/// key, does the table hold that key -- and a program can empty the result at
/// either of the last two. Only the first was guarded, which is how two
/// lifters shipped with an `fc-*` family that was silently empty (#115): the
/// microcode renders a global as `$name` and the table was keyed by the
/// resolved name, and Binary Ninja's HLIL resolves the callee before oinkie
/// sees it, so a table keyed by address matched nothing.
///
/// The two refusals are separate because they send the reader somewhere else.
/// Nothing being a call is `is_call`; nothing resolving is the symbol table or
/// `symbol_key`.
///
/// Refusing rather than returning an empty birthmark is the same judgement the
/// first of these already made: an empty one carries no evidence either way
/// and yet scores 1.0 against another empty one, so a message is strictly more
/// informative than the number.
fn refuse_an_empty_family<T: crate::Op>(p: &Program<T>) -> Result<()> {
    // One pass, returning at the first call that resolves. The count is only
    // read when none did, and reaching that answer means the whole program was
    // walked anyway -- so counting here costs a successful extraction nothing,
    // where counting first cost it a full traversal for a number it threw away.
    let mut calls = 0usize;
    for function in p.iter() {
        for op in function.iter() {
            if !op.is_call() {
                continue;
            }
            calls += 1;
            if op.symbol_key().and_then(|key| p.symbol(&key)).is_some() {
                return Ok(());
            }
        }
    }
    Err(if calls == 0 {
        Error::NoCallOperations(p.path().to_path_buf(), p.ir())
    } else {
        Error::UnresolvedCalls(p.path().to_path_buf(), p.ir(), calls)
    })
}

fn extract_function_calls<T: crate::Op>(f: &Function<T>, p: &Program<T>) -> Vec<String> {
    f.iter()
        .filter(|op| op.is_call())
        .filter_map(|op| op.symbol_key())
        .filter_map(|key| p.symbol(&key))
        .map(|s| s.to_string())
        .collect()
}

fn extract_function_calls_freq<T: crate::Op>(
    f: &Function<T>,
    p: &Program<T>,
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

    /// A program whose calls resolve to nothing, written by hand rather than
    /// lifted: the reader does not care where the JSON came from, and no real
    /// binary is needed to describe a call through a register.
    fn a_program_whose_calls_resolve_to_nothing() -> crate::program::AnyProgram {
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
        crate::program::AnyProgram::load(&path).unwrap()
    }

    /// The failure this refuses is silent: every call passes `is_call`, so the
    /// older guard let it through, and each fc-* birthmark came back empty --
    /// two of which score as a perfect match, reporting unrelated programs as
    /// identical.
    #[test]
    fn test_calls_that_resolve_to_nothing_are_refused() {
        let p = a_program_whose_calls_resolve_to_nothing();
        for bt in ["fc-set", "fc-seq", "fc-freq"] {
            let e = match Extractor::new(BirthmarkType::try_from(bt).unwrap()).extract_any(&p) {
                Err(e) => e,
                Ok(b) => panic!(
                    "{bt}: an empty birthmark was returned: {} elements",
                    b.len()
                ),
            };
            let rendered = e.to_string();
            assert!(
                matches!(e, Error::UnresolvedCalls(_, _, 2)),
                "{bt}: wrong refusal: {rendered}"
            );
            assert!(
                rendered.contains("symbol table"),
                "{bt}: does not say where to look: {rendered}"
            );
        }
    }

    /// The two refusals are not interchangeable. This one has calls; the other
    /// has none, and a reader sent to the symbol table when `is_call` is what
    /// matched nothing would look in the wrong place.
    #[test]
    fn test_the_two_empty_family_refusals_are_distinct() {
        let unresolved = a_program_whose_calls_resolve_to_nothing();
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
        let callless = crate::program::AnyProgram::load(&path).unwrap();

        let bt = BirthmarkType::try_from("fc-set").unwrap();
        let a = Extractor::new(bt.clone())
            .extract_any(&unresolved)
            .unwrap_err();
        let b = Extractor::new(bt).extract_any(&callless).unwrap_err();
        assert!(matches!(a, Error::UnresolvedCalls(_, _, _)), "{a}");
        assert!(matches!(b, Error::NoCallOperations(_, _)), "{b}");
    }

    /// The message reads the same for one call as for many.
    ///
    /// It said "1 operations are calls" for the single-call case, which is the
    /// common one: a program with one indirect call and nothing else.
    #[test]
    fn test_the_refusal_reads_correctly_for_a_single_call() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("one.json");
        std::fs::write(
            &path,
            r#"{"program":"one","path":"bin/one","ir":"ghidra-pcode",
                "symbols":{},
                "functions":[{"name":"main","ops":[
                  {"op":"CALLIND","inputs":["(register, 0x20, 8)"]}]}]}"#,
        )
        .unwrap();
        let p = crate::program::AnyProgram::load(&path).unwrap();
        let e = Extractor::new(BirthmarkType::try_from("fc-set").unwrap())
            .extract_any(&p)
            .unwrap_err()
            .to_string();
        assert!(matches!(
            Extractor::new(BirthmarkType::try_from("fc-set").unwrap()).extract_any(&p),
            Err(Error::UnresolvedCalls(_, _, 1))
        ));
        assert!(
            !e.contains("1 operations"),
            "reads as a plural for one call: {e}"
        );
        assert!(e.contains("(1 in total)"), "does not say how many: {e}");
    }

    /// The op-* families say nothing about calls, so neither refusal applies
    /// to them. A program of unresolvable calls still has operations.
    #[test]
    fn test_the_op_families_are_not_refused_for_unresolvable_calls() {
        let p = a_program_whose_calls_resolve_to_nothing();
        let b = Extractor::new(BirthmarkType::try_from("op-set").unwrap())
            .extract_any(&p)
            .expect("op-set does not depend on symbols");
        assert_eq!(b.len(), 1);
    }

    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_dest_file_name_small_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("small_test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let name = dest_file_name(&file_path).unwrap();
        assert!(name.starts_with("small_test_"));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn test_dest_file_name_large_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large_test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();

        // Write 10KB of data
        let data = vec![0u8; 10240];
        file.write_all(&data).unwrap();

        let name = dest_file_name(&file_path).unwrap();
        assert!(name.starts_with("large_test_"));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn test_get_hash_reflects_tail_difference() {
        // Two files sharing the same first 4KB but differing after it must
        // yield different hashes; otherwise their birthmark files collide.
        let dir = tempdir().unwrap();
        let path1 = dir.path().join("a.bin");
        let path2 = dir.path().join("b.bin");
        let mut data1 = vec![0u8; 5000];
        let mut data2 = vec![0u8; 5000];
        data1[4500] = 1;
        data2[4500] = 2;
        std::fs::write(&path1, &data1).unwrap();
        std::fs::write(&path2, &data2).unwrap();
        assert_ne!(get_hash(&path1).unwrap(), get_hash(&path2).unwrap());
    }

    /// The exact hash of a known input.
    ///
    /// The other hash tests say it is deterministic, that different bytes give
    /// different answers, that it is sixteen hexadecimal characters. All of
    /// those pass if the bytes come out reversed, or upper-cased, or from a
    /// different slice of the digest -- and any of those renames every
    /// birthmark ever written, which silently defeats `--skip` and orphans a
    /// directory of results.
    ///
    /// sha2 0.11 is what made that a live question: its `Output` does not
    /// implement `LowerHex`, so the hex is written out here rather than by the
    /// formatter. This is the assertion that says the rewrite kept the answer.
    ///
    /// The expected value was computed outside this crate, by hashing the same
    /// bytes with Python's hashlib, so it is not this implementation agreeing
    /// with itself.
    #[test]
    fn test_get_hash_of_a_known_input_is_exactly_this() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fixture.txt");
        std::fs::write(&path, b"oinkie hash fixture\n").unwrap();
        assert_eq!(get_hash(&path).unwrap(), "1a22831efd6c8d6e");
    }

    #[test]
    fn test_get_hash_is_deterministic() {
        let dir = tempdir().unwrap();
        let path1 = dir.path().join("a.bin");
        let path2 = dir.path().join("b.bin");
        let data = vec![7u8; 10240];
        std::fs::write(&path1, &data).unwrap();
        std::fs::write(&path2, &data).unwrap();
        assert_eq!(get_hash(&path1).unwrap(), get_hash(&path2).unwrap());
    }

    #[test]
    fn test_get_hash_io_error() {
        // Test with a non-existent file
        let path = Path::new("non_existent_file.xyz");
        let result = get_hash(path);
        assert!(result.is_err());
    }

    /// The fc-* family is built from the symbols a function calls. The call
    /// operand names its target in the lifter's own notation, which is not the
    /// form the symbol table is keyed by, so the two have to be reconciled
    /// before the lookup — otherwise every call is discarded and the birthmark
    /// comes out empty.
    #[test]
    fn test_extract_function_calls_resolves_the_symbol() {
        let program: Program<crate::ghidra::Op> =
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
            let program: Program<crate::ghidra::Op> = std::path::Path::new(fixture)
                .try_into()
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            let birthmark = Extractor::new(BirthmarkType::FcSet)
                .extract_each(&program)
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert!(
                birthmark.elements.iter().any(|e| !e.is_empty()),
                "{fixture}: every fc-set element is empty"
            );
        }
    }

    /// The danger in an fc-* birthmark is not that it is empty but that two
    /// empty ones score as a perfect match, which in a theft-detection tool
    /// reads as a positive. A program in which nothing at all is a call is
    /// far more likely to mean the lifter does not recognise its own call
    /// opcode than to mean the program makes no calls, so refuse rather than
    /// hand back something that can only mislead.
    #[test]
    fn test_fc_extraction_refuses_a_program_without_calls() {
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
        let program: Program<crate::ghidra::Op> = serde_json::from_str(json).unwrap();

        for bt in [
            BirthmarkType::FcSeq,
            BirthmarkType::FcSet,
            BirthmarkType::FcFreq,
        ] {
            let result = Extractor::new(bt.clone()).extract_each(&program);
            assert!(
                matches!(result, Err(Error::NoCallOperations(_, _))),
                "{bt}: expected a refusal, got {:?}",
                result.map(|b| b.elements.len())
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
        let program: Program<crate::ghidra::Op> = serde_json::from_str(json).unwrap();
        assert!(
            Extractor::new(BirthmarkType::OpSeq)
                .extract_each(&program)
                .is_ok()
        );
    }

    #[test]
    fn test_extractor_extract_multiple() {
        // We'll just test that Extractor::extract doesn't panic on empty input
        let extractor = Extractor::new(BirthmarkType::OpSeq);
        // Create dummy programs if possible, or just pass empty vec
        // We cannot easily create a Program here without parsing JSON, but passing empty vec works to cover line 58.
        let empty_args: Vec<&crate::program::Program<crate::ghidra::Op>> = vec![];
        let result = extractor.extract(empty_args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Pins the generated file-name layout: `{stem}_{hash}.json` with the
    /// digest truncated to a fixed width. The strip_prefix/strip_suffix pair
    /// fails if the layout changes, and the length is checked against a
    /// literal rather than HASH_PREFIX_LEN so that widening the prefix trips
    /// this test — changing it renames every birthmark file and breaks
    /// `--skip` against existing directories, which should never happen
    /// silently.
    #[test]
    fn test_dest_file_name_hash_length() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sample.bin");
        std::fs::write(&file_path, b"contents").unwrap();

        let name = dest_file_name(&file_path).unwrap();
        let hash = name
            .strip_prefix("sample_")
            .and_then(|s| s.strip_suffix(".json"))
            .expect("unexpected file name layout");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
