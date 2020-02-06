use std::fmt::{Debug, Error, Formatter};
use std::hash::Hash;
use std::mem;
use std::ops::{BitAnd, BitOr, Not, BitXor};

use itertools::Itertools;
use num_traits::{PrimInt, Zero};

pub trait BitSet: Eq + PartialEq + Ord + PartialOrd + Hash +
Copy + Clone + Debug +
BitOr<Output=Self> + BitAnd<Output=Self> + BitXor<Output=Self> + Not<Output=Self> + Zero
{
    fn size() -> usize;

    fn get(&self, index: usize) -> bool;
    fn set(&mut self, index: usize, value: bool);

    fn all_set(&self, mask: Self) -> bool {
        *self & mask == mask
    }

    fn all_ones(&self) -> bool {
        *self == !Self::zero()
    }
}

impl<T: PrimInt + Hash + Debug> BitSet for T {
    fn size() -> usize {
        Self::zero().count_zeros() as usize
    }

    fn get(&self, index: usize) -> bool {
        (*self >> index) & T::one() != T::zero()
    }

    fn set(&mut self, index: usize, value: bool) {
        let mask: T = T::one() << index;
        if value {
            *self = *self | mask;
        } else {
            *self = *self & !mask;
        }
    }
}

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
//TODO figure out a way to include the mask in this,
// so we can ignore meaningless bits without passing a mask around
// make sure this plays nicely with pmos, nmos, connect and equals
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Signal<B: BitSet> {
    low: B,
    high: B,
    strong: B,
}

impl<B: BitSet> Signal<B> {
    pub fn from_str(s: &str) -> Signal<B> {
        let mut result = Signal { low: !B::zero(), high: !B::zero(), strong: !B::zero() };

        for (i, c) in s.chars().rev().filter(|&c| c != '_').enumerate() {
            assert!(i <= B::size(), "string too large for bitset type");

            let (low, high, strong) = match c {
                '0' => (true, false, true),
                '1' => (false, true, true),
                '↓' => (true, false, false),
                '↑' => (false, true, false),
                'Z' => (false, false, false),
                c => panic!("Unexpected character '{}'", c)
            };

            result.low.set(i, low);
            result.high.set(i, high);
            result.strong.set(i, strong);
        }

        result
    }

    pub fn new(low: B, high: B, strong: B) -> Signal<B> {
        debug_assert!(low & high & !strong == B::zero(), "illegal combination 110");
        debug_assert!(!low & !high & strong == B::zero(), "illegal combination");

        Signal { low, high, strong }
    }

    pub fn connect(a: Signal<B>, b: Signal<B>) -> Option<Signal<B>> {
        let result = Signal::connect_wrap(a, b);
//        println!("Connecting {:?} and {:?} gives {:?}", a, b, result);
        result
    }

    fn connect_wrap(a: Signal<B>, b: Signal<B>) -> Option<Signal<B>> {
        let ignore = a.high & a.low & a.strong;

        if a.low & b.high != ignore { return None; };
        if a.high & b.low != ignore { return None; };

        Some(Signal::new(
            a.low | b.low,
            a.high | b.high,
            a.strong | b.strong,
        ))
    }

    pub fn pmos(gate: Signal<B>, drain: Signal<B>) -> Option<Signal<B>> {
        if gate.strong.all_ones() {
            Some(Signal::new(
                gate.low & drain.low,
                gate.low & drain.high,
                gate.low & drain.high & drain.strong,
            ))
        } else {
            None
        }
    }

    pub fn nmos(gate: Signal<B>, drain: Signal<B>) -> Option<Signal<B>> {
        if gate.strong.all_ones() {
            Some(Signal::new(
                gate.high & drain.low,
                gate.high & drain.high,
                gate.high & drain.low & drain.strong,
            ))
        } else {
            None
        }
    }

    pub fn ignored_mask(&self) -> B {
        self.low & self.high & self.strong
    }
}

impl<B: BitSet> Debug for Signal<B> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if f.alternate() {
            f.debug_struct("Signal")
                .field("low", &self.low)
                .field("high", &self.high)
                .field("strong", &self.strong)
                .finish()?
        } else {
            write!(f, "[")?;
            for index in (0..8 * mem::size_of::<B>()).rev() {
                let char = match (self.low.get(index), self.high.get(index), self.strong.get(index)) {
                    (true, false, true) => '0',
                    (false, true, true) => '1',
                    (true, false, false) => '↓',
                    (false, true, false) => '↑',
                    (false, false, false) => 'Z',
                    (true, true, true) => '.', //masked
                    _ => 'E',
                };
                write!(f, "{}", char)?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CareSignal<B: BitSet> {
    pub signal: Signal<B>,
    pub care: B
}

impl<B: BitSet> CareSignal<B> {
    pub fn new(signal: Signal<B>, care: B) -> CareSignal<B> {
        CareSignal { signal, care }
    }

    pub fn matches(&self, signal: Signal<B>) -> bool {
        (((self.signal.low ^ signal.low) & self.care) == B::zero()) &&
        (((self.signal.high ^ signal.high) & self.care) == B::zero()) &&
        (((self.signal.strong ^ signal.strong) & self.care) == B::zero())
    }
}

pub struct Query<'a, B: BitSet> {
    //signals allowed to be used as drains
    pub power: &'a [Signal<B>],
    //signals allowed to be used as gates
    pub inputs: &'a [Signal<B>],

    //expected outputs
    pub outputs: &'a [CareSignal<B>],
}

impl<'a, B: BitSet> Query<'a, B> {
    pub fn check(&self) {
        debug_assert!(
            self.power.iter()
                .chain(self.inputs.iter())
                .chain(self.outputs.iter().map(|cs| &cs.signal))
                .map(|&s| s.ignored_mask())
                .all_equal(),
            "All signals must have the same ignored mask"
        );
    }
}