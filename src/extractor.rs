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
    Ok(format!("{:x}", hash)[..HASH_PREFIX_LEN].to_string())
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
}

fn extract_birthmark_op<T: crate::Op>(p: &Program<T>, bt: &BirthmarkType) -> Result<Birthmark> {
    let now = std::time::Instant::now();
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

fn extract_function_calls<T: crate::Op>(f: &Function<T>, p: &Program<T>) -> Vec<String> {
    f.iter()
        .filter(|op| op.mnemonic() == "CALL")
        .filter_map(|op| op.inputs().first().and_then(|addr| p.symbol(addr)))
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
