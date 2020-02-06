#![allow(dead_code)]

use std::time::Instant;

use crate::pathfind::main_pathfind;
use crate::signal::{CareSignal, Query, Signal};

#[cfg(test)]
mod test;

mod signal;
mod vec_set;
mod bit;
//mod custom;
mod pathfind;

fn main() {
    //3 input NAND
    let query: Query<u8> = Query {
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
    };

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

    println!("Target output: {:?}", query.outputs[0]);
    println!("Target output: {:#?}", query.outputs[0]);
    println!("given inputs: {:?}", query.inputs);

    let start = Instant::now();

//    main_custom(&query, 6);
    main_pathfind(&query, 10);

    let end = Instant::now();
    println!("Took {}s", (end - start).as_secs_f32());
}