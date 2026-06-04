pub use crate::{Error, Result, Op, Iterable};
pub use crate::lift::{Lifter, LifterType, LifterBuilder};
pub use crate::compare::{Aggregator, Algorithm, Comparator, Comparison, PairingStrategy};
pub use crate::birthmarks::{BirthmarkType, AnalysisType, Data, Kgram, Metadata};
pub use crate::program::{Program, Function};
pub use crate::birthmarks::{Birthmark, Elements};
pub use crate::extractor::Extractor;

pub trait CsvInfo {
    fn csv_info(&self) -> String;
    fn names(&self) -> Vec<String>;
}

