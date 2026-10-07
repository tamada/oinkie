//! The progress bars the commands draw on stderr, and the one switch that
//! turns all of them off.
//!
//! A command asks [`Bars`] for its bar -- a [`Comparing`] for one that
//! compares, which also draws a line per thread and makes a [`PairBar`] per
//! pair -- and knows nothing of indicatif beyond the `ProgressBar` it is
//! handed. The library reports how far a comparison has got through
//! [`Observer`]; drawing it is this module's business alone.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use oinkie::compare::{Observer, Progress};

/// Which command a bar belongs to, which decides how it is drawn: each in a
/// colour of its own, so that `run` is told from `compare` at a glance, and
/// a pair's bar (green) from the command's.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Kind {
    Lift,
    Extract,
    Compare,
    Run,
    Review,
}

impl Kind {
    /// Every part but the message is of a fixed width, about 70 columns
    /// in all, and the message is cut at the terminal's edge: a line that
    /// wraps breaks the redrawing of every bar below it.
    fn template(self) -> String {
        let colour = match self {
            Kind::Lift => "magenta",
            Kind::Extract => "cyan",
            Kind::Compare => "yellow",
            // Orange, from the 256-colour palette: the eight basic colours
            // run out before the commands do.
            Kind::Run => "208",
            Kind::Review => "white",
        };
        format!(
            "[{{elapsed_precise}}] {{bar:30.{colour}/blue}} {{pos:>6}}/{{len:<6}} \
             eta {{eta:>3}}  {{wide_msg}}"
        )
    }
}

/// Every bar a command draws, kept together so that a pair's bar is drawn
/// below the command's rather than over it, and so that `--no-progress`
/// silences all of them in one place.
#[derive(Clone)]
pub(crate) struct Bars {
    multi: MultiProgress,
    enabled: bool,
}

impl Bars {
    /// Bars drawn on stderr, or none at all unless `enabled`. indicatif draws
    /// nothing either when stderr is not a terminal.
    pub(crate) fn new(enabled: bool) -> Bars {
        let target = if enabled {
            ProgressDrawTarget::stderr()
        } else {
            ProgressDrawTarget::hidden()
        };
        Bars {
            multi: MultiProgress::with_draw_target(target),
            enabled,
        }
    }

    /// The command's bar, counting to `len`.
    ///
    /// It redraws itself as time passes, not only as it advances: a large
    /// pair keeps it still for minutes, and its clock is what says the
    /// command is still running.
    pub(crate) fn overall(&self, kind: Kind, len: usize, message: &str) -> ProgressBar {
        let bar = ProgressBar::new(len as u64)
            .with_style(ProgressStyle::with_template(&kind.template()).unwrap())
            .with_message(message.to_string());
        let bar = self.multi.add(bar);
        if self.enabled {
            bar.enable_steady_tick(Duration::from_millis(500));
        }
        bar
    }

    /// The bars of a command that compares `pairs` pairs, its own counting
    /// to `len`. Called on the threads the pairs are compared on, whose
    /// number it reads.
    pub(crate) fn comparing(
        &self,
        kind: Kind,
        len: usize,
        pairs: usize,
        message: &str,
    ) -> Comparing {
        Comparing {
            overall: self.overall(kind, len, message),
            multi: self.enabled.then(|| self.multi.clone()),
            threads: rayon::current_num_threads(),
            slots: OnceLock::new(),
            pairs,
        }
    }
}

/// A thread's line while it compares a pair: which pair, the rows of its
/// matrix, and how long the rest will take.
const ROWS_TEMPLATE: &str =
    "  thread {prefix:>3}  {msg:14} {bar:30.green/blue} {pos:>6}/{len:<6} rows  eta {eta:>3}";

/// A thread's line while its pair is aggregated, which reports no steps to
/// count: a spinner says it is still running.
const AGGREGATING_TEMPLATE: &str = "  thread {prefix:>3}  {msg:14} {spinner:.green} aggregating...";

/// A thread's line while it compares nothing.
const IDLE_TEMPLATE: &str = "  thread {prefix:>3}  idle";

/// A comparing command's bar, and a line for each of its threads.
///
/// One line per thread, for the whole comparison, rather than one per pair:
/// the pairs come and go many times a second, and lines that came and went
/// with them would move every line below. A thread works on one pair at a
/// time, so its line shows which one.
pub(crate) struct Comparing {
    pub(crate) overall: ProgressBar,
    /// `None` under `--no-progress`, when no line is drawn.
    multi: Option<MultiProgress>,
    threads: usize,
    /// Made when the first pair starts, so that `run` shows none while it
    /// extracts.
    slots: OnceLock<Vec<Slot>>,
    pairs: usize,
}

impl Comparing {
    /// The bar of the pair at `index`, to pass as the observer of its
    /// comparison. It is drawn on the line of the thread that compares it,
    /// named by its place among the pairs: the files' names are long enough
    /// to wrap the line.
    pub(crate) fn pair(&self, index: usize) -> PairBar<'_> {
        PairBar {
            comparing: self,
            label: format!("pair {}/{}", index + 1, self.pairs),
            state: OnceLock::new(),
        }
    }

    /// Clears the threads' lines and leaves the command's bar where it ended.
    pub(crate) fn finish(&self) {
        self.overall.finish();
        if let (Some(multi), Some(slots)) = (&self.multi, self.slots.get()) {
            for slot in slots {
                slot.bar.finish_and_clear();
                multi.remove(&slot.bar);
            }
        }
    }

    /// The line of the thread this is called on.
    fn slot(&self) -> Option<&Slot> {
        let multi = self.multi.as_ref()?;
        let slots = self.slots.get_or_init(|| {
            (0..self.threads)
                .map(|i| {
                    let bar = ProgressBar::new(0).with_prefix((i + 1).to_string());
                    let slot = Slot {
                        bar: multi.add(bar),
                        pairs: Mutex::new(Vec::new()),
                    };
                    slot.show(None);
                    slot
                })
                .collect()
        });
        slots.get(rayon::current_thread_index().unwrap_or(0) % slots.len())
    }
}

/// One thread's line, and the pairs it has started and not finished.
///
/// Usually one. A thread that waits for the rows of its pair takes other
/// work meanwhile, which can be the start of another pair; the line shows
/// the latest, and the one before again once it is done.
struct Slot {
    bar: ProgressBar,
    pairs: Mutex<Vec<Arc<PairState>>>,
}

impl Slot {
    /// Draws `pair` on the line, or that the thread is idle.
    fn show(&self, pair: Option<&PairState>) {
        let style = |t| ProgressStyle::with_template(t).unwrap();
        match pair {
            None => {
                self.bar.disable_steady_tick();
                self.bar.set_style(style(IDLE_TEMPLATE));
            }
            Some(p) if p.aggregating.load(Ordering::Relaxed) => {
                self.bar.set_style(style(AGGREGATING_TEMPLATE));
                self.bar.set_message(p.label.clone());
                self.bar.enable_steady_tick(Duration::from_millis(100));
            }
            Some(p) => {
                self.bar.disable_steady_tick();
                self.bar.set_style(style(ROWS_TEMPLATE));
                self.bar.set_message(p.label.clone());
                self.bar.set_length(p.rows);
                self.bar.set_position(p.done.load(Ordering::Relaxed));
                // The estimate is the line's, and was of another pair.
                self.bar.reset_eta();
            }
        }
    }

    /// Whether `pair` is the one the line shows.
    fn shows(pairs: &[Arc<PairState>], pair: &Arc<PairState>) -> bool {
        pairs.last().is_some_and(|top| Arc::ptr_eq(top, pair))
    }
}

/// How far one pair has got.
struct PairState {
    label: String,
    rows: u64,
    done: AtomicU64,
    aggregating: AtomicBool,
}

/// One pair's bar, as [`Comparing::pair`] makes it.
pub(crate) struct PairBar<'a> {
    comparing: &'a Comparing,
    label: String,
    /// The pair, and the line it is drawn on, once its matrix has started.
    state: OnceLock<(&'a Slot, Arc<PairState>)>,
}

impl Observer for PairBar<'_> {
    fn notify(&self, progress: Progress) {
        match progress {
            Progress::MatrixStarted { rows, .. } => {
                // Reported by the thread that compares the pair, before any
                // row: the thread whose line it is drawn on.
                let Some(slot) = self.comparing.slot() else {
                    return;
                };
                let pair = Arc::new(PairState {
                    label: self.label.clone(),
                    rows: rows as u64,
                    done: AtomicU64::new(0),
                    aggregating: AtomicBool::new(false),
                });
                let mut pairs = slot.pairs.lock().unwrap();
                pairs.push(pair.clone());
                slot.show(Some(&pair));
                let _ = self.state.set((slot, pair));
            }
            Progress::RowFilled => {
                if let Some((slot, pair)) = self.state.get() {
                    pair.done.fetch_add(1, Ordering::Relaxed);
                    let pairs = slot.pairs.lock().unwrap();
                    if Slot::shows(&pairs, pair) {
                        // Read under the lock, so that the line never moves
                        // back however the rows' reports interleave.
                        slot.bar.set_position(pair.done.load(Ordering::Relaxed));
                    }
                }
            }
            Progress::Aggregating => {
                if let Some((slot, pair)) = self.state.get() {
                    pair.aggregating.store(true, Ordering::Relaxed);
                    let pairs = slot.pairs.lock().unwrap();
                    if Slot::shows(&pairs, pair) {
                        slot.show(Some(pair));
                    }
                }
            }
            _ => {}
        }
    }
}

impl Drop for PairBar<'_> {
    fn drop(&mut self) {
        if let Some((slot, pair)) = self.state.get() {
            let mut pairs = slot.pairs.lock().unwrap();
            pairs.retain(|p| !Arc::ptr_eq(p, pair));
            slot.show(pairs.last().map(Arc::as_ref));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comparing(enabled: bool) -> Comparing {
        Bars::new(enabled).comparing(Kind::Compare, 10, 10, "")
    }

    fn started(pair: &PairBar, rows: usize) {
        pair.notify(Progress::MatrixStarted { rows, columns: 1 });
    }

    /// A pair is drawn on its thread's line, which counts its rows, and the
    /// line is idle again once the pair is done.
    #[test]
    fn a_pair_is_drawn_on_its_threads_line() {
        let comparing = comparing(true);
        let pair = comparing.pair(2);
        started(&pair, 4);
        pair.notify(Progress::RowFilled);
        pair.notify(Progress::RowFilled);
        let slot = comparing.slot().unwrap();
        assert_eq!(slot.bar.message(), "pair 3/10");
        assert_eq!((slot.bar.position(), slot.bar.length()), (2, Some(4)));
        pair.notify(Progress::Aggregating);
        drop(pair);
        assert!(slot.pairs.lock().unwrap().is_empty());
    }

    /// Every thread has a line from the first pair on, whether or not it is
    /// comparing anything, so that the lines never move.
    #[test]
    fn there_is_a_line_for_every_thread() {
        let comparing = comparing(true);
        let pair = comparing.pair(0);
        started(&pair, 1);
        assert_eq!(comparing.slots.get().unwrap().len(), comparing.threads);
    }

    /// A pair started on a thread whose pair is not done is drawn until it
    /// is, and the earlier pair is drawn again after, where it has got to.
    #[test]
    fn a_line_returns_to_the_pair_it_showed_before() {
        let comparing = comparing(true);
        let outer = comparing.pair(0);
        started(&outer, 5);
        outer.notify(Progress::RowFilled);
        let inner = comparing.pair(1);
        started(&inner, 3);
        outer.notify(Progress::RowFilled);
        let slot = comparing.slot().unwrap();
        assert_eq!(slot.bar.message(), "pair 2/10");
        assert_eq!(
            slot.bar.position(),
            0,
            "the outer pair's rows are not drawn"
        );
        drop(inner);
        assert_eq!(slot.bar.message(), "pair 1/10");
        assert_eq!((slot.bar.position(), slot.bar.length()), (2, Some(5)));
    }

    /// Under `--no-progress` no line is made, however many pairs run.
    #[test]
    fn no_lines_when_disabled() {
        let comparing = comparing(false);
        let pair = comparing.pair(0);
        started(&pair, 3);
        pair.notify(Progress::RowFilled);
        pair.notify(Progress::Aggregating);
        assert!(comparing.slots.get().is_none());
    }

    /// Every bar has a template indicatif accepts, and every command's a
    /// colour no other bar has.
    #[test]
    fn each_command_draws_in_a_colour_of_its_own() {
        for t in [ROWS_TEMPLATE, AGGREGATING_TEMPLATE, IDLE_TEMPLATE] {
            ProgressStyle::with_template(t).unwrap_or_else(|e| panic!("{t}: {e}"));
        }
        let kinds = [
            Kind::Lift,
            Kind::Extract,
            Kind::Compare,
            Kind::Run,
            Kind::Review,
        ];
        let templates = kinds.map(Kind::template);
        for t in &templates {
            ProgressStyle::with_template(t).unwrap_or_else(|e| panic!("{t}: {e}"));
        }
        let colour = |t: &String| {
            let after = t.split("{bar:30.").nth(1).unwrap();
            after.split('/').next().unwrap().to_string()
        };
        let mut colours = templates.iter().map(colour).collect::<Vec<_>>();
        assert!(ROWS_TEMPLATE.contains("green/"));
        colours.push("green".to_string());
        colours.sort();
        colours.dedup();
        assert_eq!(colours.len(), kinds.len() + 1);
    }

    /// Under `--no-progress` the command's bar still counts but draws
    /// nothing.
    #[test]
    fn the_overall_bar_is_hidden_when_disabled() {
        let bar = Bars::new(false).overall(Kind::Compare, 3, "");
        bar.inc(1);
        assert_eq!(bar.position(), 1);
        assert!(bar.is_hidden());
    }
}
