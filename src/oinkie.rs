//! The value comparisons are made from, and the threads they compute on.

use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::compare::{Algorithm, Comparator};
use crate::{Error, Result};

/// Where oinkie computes: a pool of threads, and the comparators that run
/// in it.
///
/// Everything oinkie does in parallel runs on the pool of the `Oinkie` it
/// came from, never on rayon's global pool, which belongs to the
/// application. The pool's size is chosen here, in code that says so, rather
/// than by an environment variable: [`Oinkie::new`] takes one thread per
/// core, and [`Oinkie::builder`] a number of the caller's own.
///
/// Cloning is cheap and shares the pool.
///
/// ```
/// use std::num::NonZeroUsize;
///
/// use oinkie::Oinkie;
/// use oinkie::compare::Algorithm;
///
/// # fn main() -> oinkie::Result<()> {
/// let oinkie = Oinkie::builder()
///     .threads(NonZeroUsize::new(2).unwrap())
///     .build()?;
/// assert_eq!(oinkie.threads().get(), 2);
/// let comparator = oinkie.comparator(&Algorithm::Jaccard);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Oinkie {
    pool: Arc<rayon::ThreadPool>,
}

impl Oinkie {
    /// An `Oinkie` with one thread per core the machine has.
    ///
    /// Fails with [`Error::ThreadPool`] if the threads cannot be started.
    pub fn new() -> Result<Oinkie> {
        Oinkie::builder().build()
    }

    /// A builder, to choose the number of threads.
    pub fn builder() -> OinkieBuilder {
        OinkieBuilder { threads: None }
    }

    /// How many threads it computes on.
    pub fn threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.pool.current_num_threads())
            .expect("a pool is built with at least one thread")
    }

    /// The comparator that runs `algorithm`, on this `Oinkie`'s threads.
    pub fn comparator(&self, algorithm: &Algorithm) -> Comparator {
        Comparator::new(algorithm, Arc::clone(&self.pool))
    }

    /// Runs `op` on this `Oinkie`'s threads, and returns what it returns.
    ///
    /// Work `op` does in parallel with rayon runs on the same threads as the
    /// comparisons, so an application that runs its own loops over pairs or
    /// files here does not start a second pool beside oinkie's: two would
    /// together run more threads than either was given.
    pub fn install<OP, R>(&self, op: OP) -> R
    where
        OP: FnOnce() -> R + Send,
        R: Send,
    {
        self.pool.install(op)
    }
}

impl fmt::Debug for Oinkie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Oinkie")
            .field("threads", &self.threads())
            .finish()
    }
}

/// Chooses how an [`Oinkie`] is made; [`Oinkie::builder`] gives one.
#[derive(Debug, Clone)]
pub struct OinkieBuilder {
    threads: Option<NonZeroUsize>,
}

impl OinkieBuilder {
    /// Computes on `n` threads, rather than one per core.
    pub fn threads(mut self, n: NonZeroUsize) -> OinkieBuilder {
        self.threads = Some(n);
        self
    }

    /// The `Oinkie`, with its threads started.
    ///
    /// Fails with [`Error::ThreadPool`] if they cannot be.
    pub fn build(self) -> Result<Oinkie> {
        // Always a number, never rayon's default: given none, rayon reads
        // RAYON_NUM_THREADS, and the size is this builder's to say.
        let threads = self
            .threads
            .or_else(|| std::thread::available_parallelism().ok())
            .unwrap_or(NonZeroUsize::MIN);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads.get())
            .thread_name(|i| format!("oinkie-{i}"))
            .build()
            .map_err(|e| Error::ThreadPool(threads, e.to_string()))?;
        Ok(Oinkie {
            pool: Arc::new(pool),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_computes_on_as_many_threads_as_it_was_given() {
        for n in [1, 3] {
            let oinkie = Oinkie::builder()
                .threads(NonZeroUsize::new(n).unwrap())
                .build()
                .unwrap();
            assert_eq!(oinkie.threads().get(), n);
            assert_eq!(oinkie.install(rayon::current_num_threads), n);
        }
    }

    #[test]
    fn by_default_it_takes_one_thread_per_core() {
        let cores = std::thread::available_parallelism().unwrap();
        assert_eq!(Oinkie::new().unwrap().threads(), cores);
    }

    /// Two can exist side by side, each with its own threads, and a clone
    /// shares its original's.
    #[test]
    fn two_keep_their_own_threads() {
        let one = Oinkie::builder()
            .threads(NonZeroUsize::new(1).unwrap())
            .build()
            .unwrap();
        let two = Oinkie::builder()
            .threads(NonZeroUsize::new(2).unwrap())
            .build()
            .unwrap();
        assert_eq!(one.install(rayon::current_num_threads), 1);
        assert_eq!(two.install(rayon::current_num_threads), 2);
        let shared = two.clone();
        assert!(Arc::ptr_eq(&shared.pool, &two.pool));
    }
}
