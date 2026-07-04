
#[cfg(feature = "stm32h7")]
pub mod stm32;

#[cfg(feature = "lpc55")]
pub mod nxp;

#[cfg(feature = "nrf91")]
pub mod nordic;

#[cfg(feature = "renesas_ra8")]
pub mod renesas_ra8;

#[cfg(feature = "helium_simd")]
pub mod alif_m55;
