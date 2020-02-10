use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

use crate::data::undo_map::UndoMap;
use crate::signal::{BitSet, CareSignal, Query, Signal};

struct SharedState<'a, B: BitSet> {
    power_cands: &'a [Signal<B>],
    gate_cands: &'a [Signal<B>],
    outputs: &'a [CareSignal<B>],

    seen_hashes: HashSet<u64>,//, Vec<(Signal<B>, bool)>>,
//    seen: HashSet<Vec<(Signal<B>, bool)>>,
}


//TODO try entire implementation with more & instead of copying, mostly for Signals
impl<'a, B: BitSet> SharedState<'a, B> {
    fn is_done(&self, built_signals: &UndoMap<Signal<B>, bool>, depth: usize) -> Result<(), usize> {
        //TODO optimize using care mask and contains here

        //check care bits here again!

        /*if self.outputs.iter().all(|cs| built_signals.keys().any(|s| cs.matches(*s))) {
            Err(depth)
        } else {
            Ok(())
        }*/

        if self.outputs.iter().all(|cs| built_signals.contains_key(&cs.signal)) {
            Err(depth)
        } else {
            Ok(())
        }
    }

    fn recurse(&mut self, built_signals: &mut UndoMap<Signal<B>, bool>, left: usize) -> Result<(), usize> {
        self.is_done(built_signals, left)?;
        if left == 0 { return Ok(()); }
//        let _vec = built_signals.iter().map(|(&s, &b)| (s, b)).collect_vec();

//        if !self.seen.insert(vec.clone()) { return Ok(()); }

        //check is hash is unique too
        let mut hasher = FnvHasher::default();
        built_signals.hash(&mut hasher);
        let hash = hasher.finish();
        if !self.seen_hashes.insert(hash) { return Ok(()); }
//        if let Some(other) = self.seen_hashes.insert(hash, vec.clone()) {
//            println!("Found duplicate state for same hash {:?}\n{:?}\n{:?}", hash, vec.clone(), other);
//        }

        //only a single clone per step
        let mut built_signals_clone = built_signals.clone();

        //TODO try to write out separate for loops instead of chain
        for &power in self.power_cands.iter().chain(built_signals.keys()) {
            for &gate in self.gate_cands.iter().chain(built_signals.keys()) {
                self.visit_combination(power, gate, built_signals, &mut built_signals_clone, left)?
            }
        }

        Ok(())
    }

    fn visit_combination(&mut self,
                         power: Signal<B>,
                         gate: Signal<B>,
                         orig: &UndoMap<Signal<B>, bool>,
                         built_signals: &mut UndoMap<Signal<B>, bool>,
                         left: usize,
    ) -> Result<(), usize> {
        //not necessary to pick the minimum, we already know there are no smaller solutions
        //  because of iterative deepening
        self.visit_result(Signal::pmos(gate, power), orig, built_signals, left)?;
        self.visit_result(Signal::nmos(gate, power), orig, built_signals, left)?;
        Ok(())
    }

    fn visit_result(&mut self,
                    result: Option<Signal<B>>,
                    orig: &UndoMap<Signal<B>, bool>,
                    built_signals: &mut UndoMap<Signal<B>, bool>,
                    left: usize,
    ) -> Result<(), usize> {
        if let Some(result) = result {
            self.visit_as_free(result, built_signals, left)?;

            for (&other, free) in orig {
                if !free { continue; }

                let connect = Signal::connect(result, other);

                if let Some(connect) = connect {
                    built_signals.remove(other, |built_signals| {
                        self.visit_as_free(connect, built_signals, left)
                    }).unwrap_or(Ok(()))?;
                }
            }
        }

        Ok(())
    }

    fn visit_as_free(&mut self,
                     result: Signal<B>,
                     built_signals: &mut UndoMap<Signal<B>, bool>,
                     left: usize,
    ) -> Result<(), usize> {
        built_signals.insert(result, true, |built_signals| {
            self.recurse(built_signals, left - 1)
        }).unwrap_or(Ok(()))
    }
}

pub fn solve_dfs<B: BitSet>(query: &Query<B>, max_devices: usize) -> Option<usize> {
    let mut shared = SharedState {
        power_cands: query.power,
        gate_cands: query.inputs,
        outputs: query.outputs,

        //seen: Default::default(),
        seen_hashes: Default::default(),
    };

    //iterative deepening
    for depth in 0..=max_devices {
        println!("Trying depth {}", depth);
//        shared.seen.clear();
        shared.seen_hashes.clear();
        if shared.recurse(&mut Default::default(), depth).is_err() {
            return Some(depth);
        }
        println!("Saw {} distinct hashes", shared.seen_hashes.len())
    }

    None
}

//TODO big picture ideas
//   why are we constantly retrying the same combinations?
//      -> because some things might have been merged and are no longer free anymore
//   is there a way around that? some kind of "still available" matrix of combinations that don't need to be tried again?
//   maybe build path components? so if a non-free value is needed again we just add the extra cost it required?