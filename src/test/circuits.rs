use crate::pathfind::main_pathfind;
use crate::signal::{CareSignal, Query, Signal};

#[test]
fn test_single_mos() {
    let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[Signal::from_str("01")],
        outputs: &[CareSignal::new(Signal::from_str("1Z"), 0b11)],
    };

    assert_eq!(main_pathfind(&query, 8), Some(1));

    let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[Signal::from_str("01")],
        outputs: &[CareSignal::new(Signal::from_str("Z0"), 0b11)],
    };

    assert_eq!(main_pathfind(&query, 8), Some(1));
}

#[test]
fn test_not() {
    let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[Signal::from_str("01")],
        outputs: &[CareSignal::new(Signal::from_str("10"), 0b11)],
    };

    assert_eq!(main_pathfind(&query, 8), Some(2));
}

#[test]
fn test_buffer() {
    let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[Signal::from_str("01")],
        outputs: &[CareSignal::new(Signal::from_str("01"), 0b11)],
    };

    assert_eq!(main_pathfind(&query, 8), Some(4));
}

#[test]
fn test_buffer_inv_given() {
    let query = Query::<u8> {
        power: &[Signal::from_str("11"), Signal::from_str("00")],
        inputs: &[Signal::from_str("01"), Signal::from_str("10")],
        outputs: &[CareSignal::new(Signal::from_str("01"), 0b11)],
    };

    assert_eq!(main_pathfind(&query, 8), Some(2));
}

#[test]
fn test_nand2() {
    let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[
            Signal::from_str("0011"),
            Signal::from_str("0101")
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("1110"),
            0b1111,
        )],
    };

    assert_eq!(main_pathfind(&query, 8), Some(4));
}

#[test]
fn test_nor2() {
    let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[
            Signal::from_str("0011"),
            Signal::from_str("0101")
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("1000"),
            0b1111,
        )],
    };

    assert_eq!(main_pathfind(&query, 8), Some(4));
}

#[test]
#[ignore]
fn test_and2() {
    let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[
            Signal::from_str("0011"),
            Signal::from_str("0101")
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("0001"),
            0b1111,
        )],
    };

    assert_eq!(main_pathfind(&query, 8), Some(6));
}

#[test]
#[ignore]
fn test_tristate_buffer() {
    let query = Query::<u8> {
        power: &[Signal::from_str("1111"), Signal::from_str("0000")],
        inputs: &[
            Signal::from_str("0011"),
            Signal::from_str("0101")
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("0Z1Z"),
            0b1111,
        )],
    };

    assert_eq!(main_pathfind(&query, 8), Some(6));
}

#[test]
#[ignore]
fn test_nand3() {
    let query = Query::<u8> {
        power: &[Signal::from_str("1111_1111"), Signal::from_str("0000_0000")],
        inputs: &[
            Signal::from_str("0000_1111"),
            Signal::from_str("0011_0011"),
            Signal::from_str("0101_0101")
        ],
        outputs: &[CareSignal::new(
            Signal::from_str("1111_1110"),
            0b1111_1111,
        )],
    };

    assert_eq!(main_pathfind(&query, 8), Some(6));
}
