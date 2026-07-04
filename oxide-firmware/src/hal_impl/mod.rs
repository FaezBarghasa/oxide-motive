
#[cfg(any(feature = "stm32f4", feature = "stm32g4", feature = "stm32h7", feature = "stm32u5"))]
pub mod stm32;

#[cfg(any(feature = "lpc55"))]
pub mod nxp;

#[cfg(any(feature = "nrf91"))]
pub mod nordic;

#[cfg(any(feature = "renesas_ra8"))]
pub mod renesas_ra8;

#[cfg(any(feature = "alif_m55"))]
pub mod alif_m55;
