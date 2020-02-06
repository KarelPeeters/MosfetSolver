use std::fmt::{Debug, Error, Formatter};
use std::mem;

/**
Represents multiple states of a signal at once.
* low is a bitmask where a signal is pulled low
* high is a bitmask where a signal is pulled high
* strong is a bitmask where a signal is strongly pulled either high or low

All possible cases:
* strong pulldown: low=1, high=0, strong=1
* strong pullup: low=0, high=1, strong=1
* weak pulldown: low=1, high=0, strong=0
* weak pullup: low=0, high=1, strong=0
* high impedance: low=0, high=0, strong=0

*/
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
//TODO figure out a way to include the mask in this,
// so we can ignore meaningless bits without passing a mask around
// make sure this plays nicely with pmos, nmos, connect and equals
pub struct Signal {
    low: u8,
    high: u8,
    strong: u8,
}

impl Signal {
    pub fn from_str(s: &str) -> Signal {
        assert!(s.len() <= 8 * mem::size_of::<u8>());

        let mut result = Signal { low: 0, high: 0, strong: 0 };
        for (i, c) in s.chars().rev().enumerate() {
            let (low, high, strong) = match c {
                '0' => (true, false, true),
                '1' => (false, true, true),
                '↓' => (true, false, false),
                '↑' => (false, true, false),
                'Z' => (false, false, false),
                c => panic!("Unexpected character '{}'", c)
            };

            result.low |= (low as u8) << (i as u8);
            result.high |= (high as u8) << (i as u8);
            result.strong |= (strong as u8) << (i as u8);

        }
        result
    }

    pub fn new_strong(high: u8, strong: u8) -> Signal {
        Signal::new(!high & strong, high, strong)
    }

    pub fn new(low: u8, high: u8, strong: u8) -> Signal {
        debug_assert!(low & high == 0,
                      "Signal cannot be both low and high");
        debug_assert!(strong & (low | high) == strong,
                      "strong Signal must be either high or low");

        Signal { low, high, strong }
    }

    pub fn connect(a: Signal, b: Signal) -> Option<Signal> {
        if a.low & b.high != 0 { return None; };
        if a.high & b.low != 0 { return None; };

        Some(Signal::new(
            a.low | b.low,
            a.high | b.high,
            a.strong | b.strong,
        ))
    }

    pub fn pmos(gate: Signal, drain: Signal, mask: u8) -> Option<Signal> {
        if gate.strong & mask != mask {
            None
        } else {
            Some(Signal::new(
                gate.low & drain.low,
                gate.low & drain.high,
                gate.low & drain.high & drain.strong,
            ))
        }
    }

    pub fn nmos(gate: Signal, drain: Signal, mask: u8) -> Option<Signal> {
        if gate.strong & mask != mask {
            None
        } else {
            Some(Signal::new(
                gate.high & drain.low,
                gate.high & drain.high,
                gate.high & drain.low & drain.strong,
            ))
        }
    }

    pub fn equals(self: Signal, other: Signal, mask: u8) -> bool {
        (self.low & mask == other.low & mask) &&
            (self.high & mask == other.high & mask) &&
            (self.strong & mask == other.strong & mask)
    }
}

impl Debug for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[")?;
        for i in (0..8 * mem::size_of::<u8>()).rev() {
            let mask = (1 << i) as u8;
            let char = match (self.low & mask != 0, self.high & mask != 0, self.strong & mask != 0) {
                (true, false, true) => '0',
                (false, true, true) => '1',
                (true, false, false) => '↓',
                (false, true, false) => '↑',
                (false, false, false) => 'Z',
                _ => 'E',
            };
            write!(f, "{}", char)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CareSignal {
    pub signal: Signal,
    pub care: u8,
}

impl CareSignal {
    pub fn new(signal: Signal, care: u8) -> CareSignal {
        CareSignal { signal, care }
    }
}

pub struct Query<'a> {
    //mask for the relevant inputs
    pub mask: u8,

    //signals allowed to be used as drains
    pub power: &'a [Signal],
    //signals allowed to be used as gates
    pub inputs: &'a [Signal],

    pub outputs: &'a [CareSignal],
}