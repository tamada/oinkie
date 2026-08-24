pub use crate::birthmarks::{AnalysisType, BirthmarkType, Data, Kgram, Metadata};
pub use crate::birthmarks::{Birthmark, Elements};
pub use crate::compare::{
    Aggregator, Algorithm, Comparator, Comparison, PairingStrategy, escape_csv_string,
};
pub use crate::extractor::Extractor;
pub use crate::lift::{Lifter, LifterBuilder, LifterType};
pub use crate::program::{Function, Program};
pub use crate::{Error, Iterable, Op, Result};

pub trait CsvInfo {
    fn csv_info(&self) -> String;
    fn names(&self) -> Vec<String>;
}
