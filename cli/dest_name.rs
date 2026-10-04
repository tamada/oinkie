//! The name of the file an extracted birthmark is written to.
//!
//! `{original_file_stem}_{hash}.json`: a policy for where the command line puts
//! its output, so it lives with the command line rather than in the library.

use oinkie::prelude::*;
use std::path::Path;

/// Number of hex characters kept from the SHA-256 digest. 16 characters
/// is 64 bits, which puts the birthday bound around 5 billion files;
/// 8 characters (32 bits) collided at roughly 77,000.
const HASH_PREFIX_LEN: usize = 16;

/// Generates the file name for the extracted birthmark JSON file.
/// The format of resultant file name is `{original_file_stem}_{hash}.json`,
/// where `original_file_stem` is the stem of the input source file and
/// `hash` is a hash value generated from the content of the source file
/// to ensure uniqueness and avoid overwriting files with the same name.
pub(crate) fn dest_file_name(program_path: &Path) -> Result<String> {
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
