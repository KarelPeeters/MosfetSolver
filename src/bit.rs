use crate::bit::Bit::{S0, S1, W0, W1, Z};

fn print_grid() {
    let list = [Bit::S0, Bit::S1, Bit::W0, Bit::W1, Bit::Z];

    print!("\t");
    for &b in &list {
        print!("{:>5}", format!("{:?}", b));
    }
    println!();
    println!();

    for &a in &list {
        print!("{:?}\t", a);
        for &b in &list {
            let str = Bit::pmos(a, b)
                .map(|s| format!("{:?}", s))
                .unwrap_or("X".to_string());
            print!("{:>5}", str);
        }
        println!()
    }
}

#[derive(Copy, Clone, Debug)]
enum Bit {
    S0,
    S1,
    W0,
    W1,
    Z,
}

static CONNECT_TABLE: [[Option<Bit>; 5]; 5] = [
    [Some(S0), None, Some(S0), None, Some(S0)],
    [None, Some(S1), None, Some(S1), Some(S1)],
    [Some(S0), None, Some(W0), None, Some(W0)],
    [None, Some(S1), None, Some(W1), Some(W1)],
    [Some(S0), Some(S1), Some(W0), Some(W1), Some(Z)],
];

static PMOS_TABLE: [[Option<Bit>; 5]; 5] = [
    [Some(W0), Some(S1), Some(W0), Some(W1), Some(Z)],
    [Some(Z), Some(Z), Some(Z), Some(Z), Some(Z)],
    [None, None, None, None, None],
    [None, None, None, None, None],
    [None, None, None, None, None],
];

static NMOS_TABLE: [[Option<Bit>; 5]; 5] = [
    [Some(Z), Some(Z), Some(Z), Some(Z), Some(Z)],
    [Some(S0), Some(W1), Some(W0), Some(W1), Some(Z)],
    [None, None, None, None, None],
    [None, None, None, None, None],
    [None, None, None, None, None],
];

impl Bit {
    fn num(self: Bit) -> usize {
        match self {
            S0 => 0,
            S1 => 1,
            W0 => 2,
            W1 => 3,
            Z => 4,
        }
    }

    fn connect(a: Bit, b: Bit) -> Option<Bit> {
        CONNECT_TABLE[a.num()][b.num()]
    }

    fn pmos(g: Bit, d: Bit) -> Option<Bit> {
        PMOS_TABLE[g.num()][d.num()]
    }

    fn nmos(g: Bit, d: Bit) -> Option<Bit> {
        NMOS_TABLE[g.num()][d.num()]
    }
}