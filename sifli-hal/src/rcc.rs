use critical_section::CriticalSection;

pub trait SealedRccPeripheral {
    // fn frequency() -> Hertz;
    fn rcc_enable();

    fn rcc_disable();

    fn rcc_reset();
}

/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn enable_and_reset_with_cs<T: SealedRccPeripheral>(_cs: CriticalSection) {
    T::rcc_enable();
    T::rcc_reset();
}


/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn enable_and_reset<T: SealedRccPeripheral>() {
    critical_section::with(|cs| enable_and_reset_with_cs::<T>(cs));
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn disable_with_cs<T: SealedRccPeripheral>(_cs: CriticalSection) {
    T::rcc_disable();
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn disable<T: SealedRccPeripheral>() {
    critical_section::with(|cs| disable_with_cs::<T>(cs));
}