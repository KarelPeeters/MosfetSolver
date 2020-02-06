use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::env;

fn main() {
    println!("Hello world!");

    let mut state = State {
        source_candidates: HashSet::new(),
        gate_candidates: HashSet::new(),
        free_drains: HashSet::new(),
        best_solution: None,
        max_source_len: 0,
        stack: Vec::new(),
    };

    //outputs
    let targets = [0b1000];
    let max_count = 2;

    //vdd
    state.source_candidates.insert(0b_1111);

    //inputs
    state.gate_candidates.insert(0b_0011);
    state.gate_candidates.insert(0b_0101);

    state.max_source_len = state.source_candidates.len() + max_count;

    let problem = Problem {
        dead: 0,
        op: |source: i32, gate: i32| source & !gate,
        combine: |drain: i32, other: i32| drain | other,
        done: |x| targets.iter().all(|t| x.source_candidates.contains(t)),
    };

    state.brute_force(&problem, 0);
    match state.best_solution {
        None => {
            println!("No solution found")
        }
        Some(vec) => {
            println!("Found solution {:?}", vec)
        }
    }

    assert!(state.stack.is_empty())
}

#[derive(Debug)]
struct State<S: Hash + Eq + Copy> {
    source_candidates: HashSet<S>,
    gate_candidates: HashSet<S>,
    free_drains: HashSet<S>,

    best_solution: Option<Vec<S>>,
    max_source_len: usize,

    //tmp fields
    stack: Vec<(usize, HashSet<S>, HashSet<S>, HashSet<S>)>,
}

//TODO fix determinism,
// neither NOR nor NAND seems to work every time
struct Problem<S, O: Fn(S, S) -> S, C: Fn(S, S) -> S, D: Fn(&State<S>) -> bool> {
    dead: S,

    op: O,
    combine: C,
    done: D,
}

impl<S: Hash + Eq + Copy + Debug + num::Zero> State<S> {
    fn push(&mut self, id: usize) {
        self.stack.push((id, self.source_candidates.clone(), self.gate_candidates.clone(), self.free_drains.clone()));

//        println!("{:?}", self.stack);
    }

    fn pop(&mut self, id: usize) {
        match self.stack.pop() {
            Some((i, s, g, f)) => {
                assert_eq!(i, id, "id mismatch");
                assert_eq!(s, self.source_candidates, "source mismatch");
                assert_eq!(g, self.gate_candidates, "gate mismatch");
                assert_eq!(f, self.free_drains, "free mismatch");
            }
            None => panic!("Stack empty")
        }
//        println!("{:?}", self.stack);
    }

    fn found_solution(&mut self) {
        //if shorter or no solution yet
        if self.best_solution.is_none() || self.source_candidates.len() < self.max_source_len {
            let solution = self.source_candidates.intersection(&self.gate_candidates).copied().collect();
            self.best_solution = Some(solution);
            self.max_source_len = self.source_candidates.len();
        }
    }

    fn brute_force<
        O: Fn(S, S) -> S,
        C: Fn(S, S) -> S,
        D: Fn(&Self) -> bool
    >(
        &mut self,
        problem: &Problem<S, O, C, D>,
        count: usize,
    ) {
        self.push(0);

        if count > self.max_source_len {
            self.pop(0);
            return;
        }

        println!("Checking if solution, {:?}", self);
        if (problem.done)(self) {
            self.found_solution();
            self.pop(0);
            return;
        }

        let source_candidates: Vec<S> = self.source_candidates.iter().copied().collect();
        let gate_candidates: Vec<S> = self.gate_candidates.iter().copied().collect();
        let free_drains: Vec<S> = self.free_drains.iter().copied().collect();

        for &source in &source_candidates {
            self.push(1);
            let source_was_free = self.free_drains.remove(&source);

            for &gate in &gate_candidates {
                self.push(2);
                let gate_was_free = self.free_drains.remove(&gate);

                let drain = (problem.op)(source, gate);

                if drain == problem.dead {
                    self.pop(2);
                    continue;
                }

                self.push(3);
                //try adding as free drain
                if self.free_drains.insert(drain) {
                    let inserted_as_source = self.source_candidates.insert(drain);
                    let inserted_as_gate = self.gate_candidates.insert(drain);

                    self.brute_force(problem, count + 1);

                    self.free_drains.remove(&drain);
                    if inserted_as_source { self.source_candidates.remove(&drain); }
                    if inserted_as_gate { self.gate_candidates.remove(&drain); }
                }
                self.pop(3);

                //try combining with other free drains
                self.push(4);
                for &other_drain in &free_drains {
                    let combined = (problem.combine)(drain, other_drain);

                    if self.free_drains.insert(combined) {
//                        assert!(self.free_drains.remove(&other_drain));
                        self.free_drains.remove(&other_drain);
                        let existed_as_source = self.source_candidates.remove(&combined);
                        let existed_as_gate = self.gate_candidates.remove(&combined);

                        self.brute_force(problem, count + 1);

                        if existed_as_source { self.source_candidates.insert(combined); }
                        if existed_as_gate { self.gate_candidates.insert(combined); }

                        self.free_drains.remove(&combined);
                        self.free_drains.insert(other_drain);
                    }
                }
                self.pop(4);

                if gate_was_free { self.free_drains.insert(gate); }
                self.pop(2);
            }

            if source_was_free { self.free_drains.insert(source); }
            self.pop(1);
        }

        self.pop(0);

        assert_eq!(HashSet::from_iter(source_candidates.iter().copied()), self.source_candidates);
        assert_eq!(HashSet::from_iter(gate_candidates.iter().copied()), self.gate_candidates);
        assert_eq!(HashSet::from_iter(free_drains.iter().copied()), self.free_drains);
    }
}