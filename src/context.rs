use std::{path::PathBuf, sync::LazyLock};
use crate::prelude::*;

pub enum Progressor {
    S(Simple),
    N(),
}

impl Progressor {
    pub fn inc_with(&self, delta: u64) {
        match self {
            Progressor::S(s) => s.inc_with(delta),
            Progressor::N() => (),
        }
    }

    pub fn inc(&self) {
        self.inc_with(1);
    }

    pub fn println<S: Into<String>>(&self, message: S) {
        match self {
            Progressor::S(s) => s.p.println(message.into()),
            Progressor::N() => (),
        }
    }
}

pub struct Simple {
    p: indicatif::ProgressBar,
}

impl Simple {
    fn new(p: indicatif::ProgressBar) -> Self {
        Self { p }
    }

    fn inc_with(&self, delta: u64) {
        self.p.inc(delta);
    }
}

pub struct Context {
    args: Vec<PathBuf>,
    algorithm: AnalysisType,
    dest: Option<PathBuf>,
    show_progress: bool,
    strategy: PairingStrategy,
    progress: LazyLock<indicatif::MultiProgress>,
}

impl Context {
    pub fn new(strategy: PairingStrategy, algorithm: AnalysisType, args: Vec<PathBuf>) -> Self {
        Self::new_with(strategy, algorithm, args, None, false)
    }

    pub fn new_with(strategy: PairingStrategy, algorithm: AnalysisType, args: Vec<PathBuf>, dest: Option<PathBuf>, show_progress: bool) -> Self {
        Self { args, algorithm, dest, show_progress, strategy, progress: LazyLock::new(indicatif::MultiProgress::new) }
    }

    pub fn dest(&self) -> Result<Box<dyn std::io::Write>> {
        if let Some(dest) = &self.dest {
            std::fs::create_dir_all(dest)
                .map_err(Error::Io)?;
            std::fs::File::create(dest.join("results.csv"))
                .map_err(Error::Io)
                .map(|f| Box::new(f) as Box<dyn std::io::Write>)
        } else {
            Ok(Box::new(std::io::stdout()))
        }
    }

    pub fn comparator(&self) -> &Comparator {
        self.algorithm.comparator()
    }

    pub fn open_dest(&self, index: usize) -> Result<Box<dyn std::io::Write>> {
        if let Some(dest) = &self.dest {
            std::fs::File::create(dest.join(format!("{index:05}.csv")))
                .map_err(Error::Io)
                .map(|f| Box::new(f) as Box<dyn std::io::Write>)
        } else {
            Ok(Box::new(std::io::sink()))
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &PathBuf)> + Send + '_> {
        self.strategy.pairs(&self.args)
    }

    pub fn main_progressor(&self) -> Progressor {
        if self.show_progress {
            let count = self.strategy.compare_count(&self.args);
            let p = indicatif::ProgressBar::new(count as u64)
                .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap())
                .with_message("Processing files...")
                .with_position(0);
            let p = self.progress.add(p);
            Progressor::S(Simple::new(p))
        } else {
            Progressor::N()
        }
    }

    pub fn sub_progressor<S: Into<String>>(&self, message: S, total: u64) -> Progressor {
        if self.show_progress {
            let p = indicatif::ProgressBar::new(total)
                .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap())
                .with_message(message.into())
                .with_position(0);
            self.progress.add(p.clone());
            Progressor::S(Simple::new(p))
        } else {
            Progressor::N()
        }
    }

    pub fn remove_progress(&self, p: Progressor) {
        if self.show_progress && let Progressor::S(s) = p {
            self.progress.remove(&s.p);
        }
    }
}
