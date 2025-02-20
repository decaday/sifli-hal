use embassy_hal_internal::Peripheral;
// use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt;
use crate::rcc::SealedRccEnableReset;

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
    /// Channel 3.
    Ch3,
    /// Channel 4.
    Ch4,
    /// Channel 5, only available on ATIM.
    Ch5,
    /// Channel 6, only available on ATIM.
    Ch6,
}

impl Channel {
    /// Get the channel index (0..3)
    pub fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
            Channel::Ch5 => 4,
            Channel::Ch6 => 5,
        }
    }
}

/// Amount of bits of a timer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerBits {
    /// 16 bits.
    Bits16,
    /// 32 bits.
    Bits32,
}

// struct State {
//     up_waker: AtomicWaker,
//     cc_waker: [AtomicWaker; 4],
// }

// impl State {
//     const fn new() -> Self {
//         Self {
//             up_waker: AtomicWaker::new(),
//             cc_waker: [const { AtomicWaker::new() }; 4],
//         }
//     }
// }

trait SealedInstance: SealedRccEnableReset + Peripheral<P = Self> {
    // /// Async state for this timer
    // fn state() -> &'static State;
}

/// timer instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static {
    /// Interrupt for this timer.
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Amount of bits this timer has.
    const BITS: TimerBits;

    /// Registers for this timer.
    ///
    /// This is a raw pointer to the register block. The actual register block layout varies depending on the timer type.
    fn regs() -> *mut ();
}

pub trait AtimInstance: Instance + 'static {}
pub trait GptimInstance: Instance + 'static {}
pub trait BimInstance: Instance + 'static {}



// TODO: move to _generated.rs
use crate::peripherals;

impl SealedInstance for peripherals::ATIM1 {}
impl Instance for peripherals::ATIM1 {
    type Interrupt = interrupt::typelevel::ATIM1;
    const BITS: TimerBits = TimerBits::Bits32;
    fn regs() -> *mut () {
        crate::pac::ATIM1.as_ptr()
    }
}
impl AtimInstance for peripherals::ATIM1 {}

impl SealedInstance for peripherals::GPTIM1 {}
impl Instance for peripherals::GPTIM1 {
    type Interrupt = interrupt::typelevel::GPTIM1;
    const BITS: TimerBits = TimerBits::Bits32;
    fn regs() -> *mut () { 
        crate::pac::GPTIM1.as_ptr()
    }
}
impl GptimInstance for peripherals::GPTIM1 {}

impl SealedInstance for peripherals::GPTIM2 {}
impl Instance for peripherals::GPTIM2 {
    type Interrupt = interrupt::typelevel::GPTIM2;
    const BITS: TimerBits = TimerBits::Bits32;
    fn regs() -> *mut () { 
        crate::pac::GPTIM2.as_ptr()
    }
}
impl GptimInstance for peripherals::GPTIM2 {}

impl SealedInstance for peripherals::BTIM1 {}
impl Instance for peripherals::BTIM1 {
    type Interrupt = interrupt::typelevel::BTIM1;
    const BITS: TimerBits = TimerBits::Bits32;
    fn regs() -> *mut () { 
        crate::pac::BTIM1.as_ptr()
    }
}
impl BimInstance for peripherals::BTIM1 {}

impl SealedInstance for peripherals::BTIM2 {}
impl Instance for peripherals::BTIM2 {
    type Interrupt = interrupt::typelevel::BTIM2;
    const BITS: TimerBits = TimerBits::Bits32;
    fn regs() -> *mut () { 
        crate::pac::BTIM2.as_ptr()
    }
}
impl BimInstance for peripherals::BTIM2 {}
