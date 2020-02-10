#![allow(dead_code)]
#![feature(const_fn)]
#![feature(try_trait)]
#![feature(bool_to_option)]

use std::time::Instant;

use crate::signal::{BitSet, CareSignal, Query, Signal};
use crate::solvers::pathfind::solve_bfs;
use crate::solvers::dfs::solve_dfs;

#[cfg(test)]
mod test;

mod signal;
mod data;
mod bit;
mod solvers;

fn main() {
    //3 input NAND
    /*let query: Query<u8> = Query {
        power: &[
            Signal::from_str("1111_1111"), //Vcc
            Signal::from_str("0000_0000"), //Gnd
        ],
        inputs: &[
            Signal::from_str("0000_1111"),
            Signal::from_str("0011_0011"),
            Signal::from_str("0101_0101"),
        ],
        outputs: &[
            CareSignal::new(
                Signal::from_str("1111_1110"),
                0b1111_1111,
            ),
        ],
    };*/

    //inverting/non-inverting buffer
    /*let query: Query<u8> = Query {
        power: &[
            Signal::from_str("1111"), //Vcc
            Signal::from_str("0000"), //Gnd
        ],
        inputs: &[
            Signal::from_str("0011"), //in1
            Signal::from_str("0101"), //en1
        ],
        outputs: &[
            CareSignal::new(
                Signal::from_str("Z0Z1"),
                0b1111,
            ),
        ],
    };*/

    //simple buffer gate
    /*let query: Query<u8> = Query {
        power: &[
            Signal::from_str("11"), //Vcc
            Signal::from_str("00"), //Gnd
        ],
        inputs: &[
            Signal::from_str("01"),
        ],
        outputs: &[
            CareSignal::new(
                Signal::from_str("01"),
                0b11,
            ),
        ],
    };*/

    //xor2
    /*let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[
            Signal::from_str("0011"),
            Signal::from_str("0101"),
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("0110"),
            0b1111,
        )],
    };*/

    //not
    /*let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[
            Signal::from_str("01"),
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("10"),
            0b11,
        )],
    };*/

    //half adder
    let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[Signal::from_str("0011"), Signal::from_str("0101")],
        outputs: &[
            CareSignal::new(Signal::from_str("0110"), 0b1111),
            CareSignal::new(Signal::from_str("0001"), 0b1111),
        ],
    };

    println!("Target output: {:?}", query.outputs[0]);
    println!("Target output: {:#?}", query.outputs[0]);
    println!("given inputs: {:?}", query.inputs);

    let start = Instant::now();

//    main_custom(&query, 6);

    //TODO read blogpost about compressing bfs
    let result = solve(&query, 15);
    println!("Solved with result {:?}", result);

    let end = Instant::now();
    println!("Took {}s", (end - start).as_secs_f32());
}

fn solve<B: BitSet>(query: &Query<B>, max_devices: usize) -> Option<usize> {
    query.check();
    solve_dfs(query, max_devices)
}
