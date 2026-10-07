//! The library used the way an outside crate uses it: through the public API
//! alone, with no knowledge of which tool lifted a program.

use oinkie::birthmarks::BirthmarkType;
use oinkie::compare::{Aggregator, Algorithm, PairingStrategy};
use oinkie::extract::Extractor;
use oinkie::{Oinkie, Program};
use std::path::Path;

fn load_program(path: &str) -> Program {
    Program::load(Path::new(path)).unwrap()
}

#[test]
fn test_extractor_and_comparator() {
    let p1 = load_program("testdata/lifted/pcodes/hello_clang.json");
    let p2 = load_program("testdata/lifted/pcodes/hello_gcc.json");

    let combinations = vec![
        (BirthmarkType::OpSeq, Algorithm::Jaccard),
        (BirthmarkType::OpSeq, Algorithm::Dice),
        (BirthmarkType::OpSeq, Algorithm::Simpson),
        (BirthmarkType::OpSeq, Algorithm::Levenshtein),
        (BirthmarkType::OpSeq, Algorithm::Lcs),
        (BirthmarkType::OpSet, Algorithm::Jaccard),
        (BirthmarkType::OpFreq, Algorithm::Cosine),
        (BirthmarkType::OpFreq, Algorithm::Euclidean),
        (BirthmarkType::OpFreq, Algorithm::WeightedJaccard),
        (BirthmarkType::OpFreq, Algorithm::JensenShannon),
        (BirthmarkType::OpFreq, Algorithm::Tanimoto),
        (BirthmarkType::OpKgramFreq(3), Algorithm::JensenShannon),
        (BirthmarkType::FcFreq, Algorithm::Tanimoto),
        (BirthmarkType::OpKgramSeq(2), Algorithm::Jaccard),
        (BirthmarkType::OpKgramSet(3), Algorithm::Dice),
        (BirthmarkType::OpKgramFreq(2), Algorithm::Cosine),
        (BirthmarkType::FcSeq, Algorithm::Levenshtein),
        (BirthmarkType::FcSet, Algorithm::Jaccard),
        (BirthmarkType::FcFreq, Algorithm::Cosine),
    ];

    for (bt, algo) in combinations {
        let extractor = Extractor::new(bt.clone());
        let b1 = extractor.extract(&p1).unwrap();
        let b2 = extractor.extract(&p2).unwrap();

        let comparator = Oinkie::new().unwrap().comparator(&algo);

        // Test with Hungarian aggregator
        let result_hungarian = comparator
            .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian)
            .unwrap();
        assert!(
            result_hungarian.similarity() >= -0.01 && result_hungarian.similarity() <= 1.01,
            "Similarity out of bounds for {:?} and {:?} with Hungarian: {}",
            bt,
            algo,
            result_hungarian.similarity()
        );

        // Test with TopN aggregator
        use std::str::FromStr;
        let result_topn = comparator
            .compare_birthmarks(&b1, &b2, &Aggregator::from_str("topn:all").unwrap())
            .unwrap();
        assert!(
            result_topn.similarity() >= -0.01 && result_topn.similarity() <= 1.01,
            "Similarity out of bounds for {:?} and {:?} with TopN: {}",
            bt,
            algo,
            result_topn.similarity()
        );
    }
}

#[test]
fn test_pairing_strategies() {
    let p1 = load_program("testdata/lifted/pcodes/hello_clang.json");
    let p2 = load_program("testdata/lifted/pcodes/hello_gcc.json");
    let programs = vec![p1, p2];

    assert_eq!(PairingStrategy::AllAndSelf.compare_count(&programs), 3);
    assert_eq!(PairingStrategy::AllAndSelf.pairs(&programs).count(), 3);

    assert_eq!(PairingStrategy::All.compare_count(&programs), 1);
    assert_eq!(PairingStrategy::All.pairs(&programs).count(), 1);

    assert_eq!(PairingStrategy::SelfCoverage.compare_count(&programs), 2);
    assert_eq!(PairingStrategy::SelfCoverage.pairs(&programs).count(), 2);

    assert_eq!(PairingStrategy::Adjacent.compare_count(&programs), 1);
    assert_eq!(PairingStrategy::Adjacent.pairs(&programs).count(), 1);

    assert_eq!(PairingStrategy::FirstVsOthers.compare_count(&programs), 1);
    assert_eq!(PairingStrategy::FirstVsOthers.pairs(&programs).count(), 1);

    assert_eq!(PairingStrategy::LastVsOthers.compare_count(&programs), 1);
    assert_eq!(PairingStrategy::LastVsOthers.pairs(&programs).count(), 1);
}

#[test]
fn test_empty_comparisons() {
    // Tests what happens when programs have 0 elements
    // This requires creating an empty program or mock if possible
    // We can also test mismatched types
    let p1 = load_program("testdata/lifted/pcodes/hello_clang.json");
    let ext1 = Extractor::new(BirthmarkType::OpSeq);
    let ext2 = Extractor::new(BirthmarkType::FcSeq);

    let b1 = ext1.extract(&p1).unwrap();
    let b2 = ext2.extract(&p1).unwrap();

    let comparator = Oinkie::new().unwrap().comparator(&Algorithm::Jaccard);
    let result = comparator.compare_birthmarks(&b1, &b2, &Aggregator::Hungarian);
    assert!(result.is_err()); // Mismatched types
}
