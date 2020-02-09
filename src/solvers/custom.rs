use smallset::SmallSet;

use crate::signal::{CareSignal, Query, Signal, BitSet};

/*
TODO:

design datastructure, possibly using sets? or something more bitset-like?
write functions to take and put in signals, make everything reversible

actually write the algorithm

try to alternate adding pmos and nmos, solutions probably have a ratio around 50/50

*/

fn find_solution<B: BitSet>(query: &Query<B>, max_gates: usize) {
    let mut state = State {
        power_signals: SmallSet::new(),
        gate_signals: SmallSet::new(),
        free_signals: SmallSet::new(),
        outputs: query.outputs,
        max_gates,
    };

    for &s in query.power {
        state.power_signals.insert(s);
    }
    for &s in query.inputs {
        state.gate_signals.insert(s);
    }

    //TODO decide whether to do this or from_iter instead
//    state.power_signals.extend(query.power.iter().copied());
//    state.gate_signals.extend(query.inputs.iter().copied());
    //TODO decide whether to add power to gate or not
//    state.gate_signals.extend(query.power.iter().copied());


    return state.recurse(0);
}

#[derive(Debug)]
struct State<'a, B: BitSet> {
    power_signals: SmallSet<[Signal<B>; 10]>,
    gate_signals: SmallSet<[Signal<B>; 10]>,
    free_signals: SmallSet<[Signal<B>; 10]>,

    outputs: &'a [CareSignal<B>],
    max_gates: usize,
}

impl<'a, B: BitSet> State<'a, B> {
    fn recurse(&mut self, i: usize) {
        self.check_solution();
        if i == self.max_gates { return; }

        let power_signals: Vec<Signal<B>> = self.power_signals.iter().copied().collect();
        let gate_signals: Vec<Signal<B>> = self.gate_signals.iter().copied().collect();
        let free_signals: Vec<Signal<B>> = self.free_signals.iter().copied().collect();

        for power in power_signals.iter().copied() {
//            println!("{:-<2$}Picking power {:?}", "", power, i);

            let power_was_free = self.free_signals.remove(&power);

            for gate in gate_signals.iter().copied() {
//                println!("{:-<2$}Picking gate {:?}", "", gate, i);

                /*let vcc = Signal::new_strong(0b1111_1111, 0b1111_1111);
                let gnd = Signal::new_strong(0b0000_0000, 0b1111_1111);
                let a = Signal::new_strong(0b0000_1111, 0b1111_1111);
                let b = Signal::new_strong(0b0011_0011, 0b1111_1111);
                let c = Signal::new_strong(0b0101_0101, 0b1111_1111);

                if i == 0 && power == vcc && gate == a {
                    println!("0")
                }
                if i == 1 && power == vcc && gate == b {
                    println!("1")
                }
                if i == 2 && power == vcc && gate == c {
                    println!("2")
                }
                if i == 3 && power == gnd && gate == a {
                    println!("3")
                }
                if i == 4 && power == Signal::nmos(a, gnd, self.mask).unwrap() && gate == b {
                    println!("4")
                }
                if i == 5 && power == Signal::nmos(b, Signal::nmos(a, gnd, self.mask).unwrap(), self.mask).unwrap() && gate == c {
                    println!("5")
                }*/

                let gate_was_free = self.free_signals.remove(&gate);

                //alternate order of attempting pmos/nmos
                if i % 2 == 0 {
                    self.add_device(Signal::pmos(gate, power), &free_signals, i);
                    self.add_device(Signal::nmos(gate, power), &free_signals, i);
                } else {
                    self.add_device(Signal::nmos(gate, power), &free_signals, i);
                    self.add_device(Signal::pmos(gate, power), &free_signals, i);
                }

                if gate_was_free {
                    assert!(self.free_signals.insert(gate));
                }
            }

            if power_was_free {
                assert!(self.free_signals.insert(power));
            }

//            debug_assert_eq!(self.power_signals, power_signals.iter().copied().collect());
//            debug_assert_eq!(self.gate_signals, gate_signals.iter().copied().collect());
//            debug_assert_eq!(self.free_signals, free_signals.iter().copied().collect());
        }
    }

    fn check_solution(&self) {
        for &CareSignal { signal, care } in self.outputs {
            if care == self.mask {
                //quick way out, just find signal that's entirely equal
                if !self.power_signals.contains(&signal) { return; }
            } else {
                //need to check equality taking into account the mask
                if !self.power_signals.iter()
                    .any(|s| s.equals(signal, care)) { return; }
            }
        }

        //at this point we've found a solution
        println!("Found solution, {:?}", self);
    }

    fn add_device(&mut self, result: Option<Signal<B>>, free_signals: &Vec<Signal<B>>, i: usize) {
        if let Some(new) = result {
            //add as new free signal
            self.add_as_free(new, i);

            //TODO is merging with a single one enough? try a 3-input NAND gate
            //merge with other free signal
            for other in free_signals.iter().copied() {
                //free signal could have been used to make this one, check again
                if self.free_signals.remove(&other) {
                    if let Some(combined) = Signal::connect(new, other) {
                        self.add_as_free(combined, i);
                    }
                    assert!(self.free_signals.insert(other))
                }
            }
        }
    }

    fn add_as_free(&mut self, new: Signal<B>, i: usize) {
        if self.free_signals.insert(new) {
            let power_is_new = self.power_signals.insert(new);
            let gate_is_new = self.gate_signals.insert(new);

            self.recurse(i + 1);

            if power_is_new { assert!(self.power_signals.remove(&new)) }
            if gate_is_new { assert!(self.gate_signals.remove(&new)) }
            assert!(self.free_signals.remove(&new));
        }
    }
}

pub fn main_custom<B: BitSet>(query: &Query<B>, max_gates: usize) {
    let result = find_solution(&query, max_gates);
    println!("{:?}", result);
}