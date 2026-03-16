use clap::ValueEnum;
use oinkie::prelude::BirthmarkType;

#[derive(Debug, clap::Parser, ValueEnum, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum BType {
    /// the sequence of method calls in a program.
    FcSeq,
    /// the frequency of method calls in a program.
    FcFreq,
    /// the Set of method calls in a program.
    FcSet,
    /// the sequence of operations in a program.
    OpSeq,
    /// the set of operations in a program.
    OpSet,
    /// the frequency of operations in a program.
    OpFreq,
    /// the sequence of 1-grams of operations in a program.
    OpUniGramSeq,
    /// the sequence of 2-grams of operations in a program.
    OpBiGramSeq,
    /// the sequence of 3-grams of operations in a program.
    OpTriGramSeq,
    /// the sequence of 4-grams of operations in a program.
    OpQuadGramSeq,
    /// the sequence of 5-grams of operations in a program.
    OpPentaGramSeq,
    /// the sequence of 6-grams of operations in a program.
    OpHexaGramSeq,
    /// the sequence of 7-grams of operations in a program.
    OpHeptaGramSeq,
    /// the sequence of 8-grams of operations in a program.
    OpOctaGramSeq,
    /// the frequency of 1-grams of operations in a program.
    OpUniGramFreq,
    /// the frequency of 2-grams of operations in a program.
    OpBiGramFreq,
    /// the frequency of 3-grams of operations in a program.
    OpTriGramFreq,
    /// the frequency of 4-grams of operations in a program.
    OpQuadGramFreq,
    /// the frequency of 5-grams of operations in a program.
    OpPentaGramFreq,
    /// the frequency of 6-grams of operations in a program.
    OpHexaGramFreq,
    /// the frequency of 7-grams of operations in a program.
    OpHeptaGramFreq,
    /// the frequency of 8-grams of operations in a program.
    OpOctaGramFreq,
    /// the set of 1-grams of operations in a program.
    OpUniGramSet,
    /// the set of 2-grams of operations in a program.
    OpBiGramSet,
    /// the set of 3-grams of operations in a program.
    OpTriGramSet,
    /// the set of 4-grams of operations in a program.
    OpQuadGramSet,
    /// the set of 5-grams of operations in a program.
    OpPentaGramSet,
    /// the set of 6-grams of operations in a program.
    OpHexaGramSet,
    /// the set of 7-grams of operations in a program.
    OpHeptaGramSet,
    /// the set of 8-grams of operations in a program.
    OpOctaGramSet,
}

impl std::fmt::Display for BType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BType::FcSeq => write!(f, "fc-seq"),
            BType::FcFreq => write!(f, "fc-freq"),
            BType::FcSet => write!(f, "fc-set"),
            BType::OpSeq => write!(f, "op-seq"),
            BType::OpSet => write!(f, "op-set"),
            BType::OpFreq => write!(f, "op-freq"),
            BType::OpUniGramSeq => write!(f, "op-1gram-seq"),
            BType::OpBiGramSeq => write!(f, "op-2gram-seq"),
            BType::OpTriGramSeq => write!(f, "op-3gram-seq"),
            BType::OpQuadGramSeq => write!(f, "op-4gram-seq"),
            BType::OpPentaGramSeq => write!(f, "op-5gram-seq"),
            BType::OpHexaGramSeq => write!(f, "op-6gram-seq"),
            BType::OpHeptaGramSeq => write!(f, "op-7gram-seq"),
            BType::OpOctaGramSeq => write!(f, "op-8gram-seq"),
            BType::OpUniGramFreq => write!(f, "op-1gram-freq"),
            BType::OpBiGramFreq => write!(f, "op-2gram-freq"),
            BType::OpTriGramFreq => write!(f, "op-3gram-freq"),
            BType::OpQuadGramFreq => write!(f, "op-4gram-freq"),
            BType::OpPentaGramFreq => write!(f, "op-5gram-freq"),
            BType::OpHexaGramFreq => write!(f, "op-6gram-freq"),
            BType::OpHeptaGramFreq => write!(f, "op-7gram-freq"),
            BType::OpOctaGramFreq => write!(f, "op-8gram-freq"),
            BType::OpUniGramSet => write!(f, "op-1gram-set"),
            BType::OpBiGramSet => write!(f, "op-2gram-set"),
            BType::OpTriGramSet => write!(f, "op-3gram-set"),
            BType::OpQuadGramSet => write!(f, "op-4gram-set"),
            BType::OpPentaGramSet => write!(f, "op-5gram-set"),
            BType::OpHexaGramSet => write!(f, "op-6gram-set"),
            BType::OpHeptaGramSet => write!(f, "op-7gram-set"),
            BType::OpOctaGramSet => write!(f, "op-8gram-set"),
        }
    }
}

impl From<BType> for BirthmarkType {
    fn from(value: BType) -> Self {
        match value {
            BType::FcSeq => BirthmarkType::FcSeq,
            BType::FcFreq => BirthmarkType::FcFreq,
            BType::FcSet => BirthmarkType::FcSet,
            BType::OpSeq => BirthmarkType::OpSeq,
            BType::OpSet => BirthmarkType::OpSet,
            BType::OpFreq => BirthmarkType::OpFreq,
            BType::OpUniGramSeq => BirthmarkType::OpKgramSeq(1),
            BType::OpBiGramSeq => BirthmarkType::OpKgramSeq(2),
            BType::OpTriGramSeq => BirthmarkType::OpKgramSeq(3),
            BType::OpQuadGramSeq => BirthmarkType::OpKgramSeq(4),
            BType::OpPentaGramSeq => BirthmarkType::OpKgramSeq(5),
            BType::OpHexaGramSeq => BirthmarkType::OpKgramSeq(6),
            BType::OpHeptaGramSeq => BirthmarkType::OpKgramSeq(7),
            BType::OpOctaGramSeq => BirthmarkType::OpKgramSeq(8),
            BType::OpUniGramFreq => BirthmarkType::OpKgramFreq(1),
            BType::OpBiGramFreq => BirthmarkType::OpKgramFreq(2),
            BType::OpTriGramFreq => BirthmarkType::OpKgramFreq(3),
            BType::OpQuadGramFreq => BirthmarkType::OpKgramFreq(4),
            BType::OpPentaGramFreq => BirthmarkType::OpKgramFreq(5),
            BType::OpHexaGramFreq => BirthmarkType::OpKgramFreq(6),
            BType::OpHeptaGramFreq => BirthmarkType::OpKgramFreq(7),
            BType::OpOctaGramFreq => BirthmarkType::OpKgramFreq(8),
            BType::OpUniGramSet => BirthmarkType::OpKgramSet(1),
            BType::OpBiGramSet => BirthmarkType::OpKgramSet(2),
            BType::OpTriGramSet => BirthmarkType::OpKgramSet(3),
            BType::OpQuadGramSet => BirthmarkType::OpKgramSet(4),
            BType::OpPentaGramSet => BirthmarkType::OpKgramSet(5),
            BType::OpHexaGramSet => BirthmarkType::OpKgramSet(6),
            BType::OpHeptaGramSet => BirthmarkType::OpKgramSet(7),
            BType::OpOctaGramSet => BirthmarkType::OpKgramSet(8),
        }
    }
}
