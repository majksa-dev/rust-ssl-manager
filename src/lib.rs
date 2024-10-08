//! Example of a simple library
//!
//! # Examples
//!
//! ```
//! let result = add(2, 2);
//! assert_eq!(result, 4);
//! ```

#[cfg(feature = "acme")]
pub mod acme;
mod certificate;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "acme")]
pub mod dns;

pub use certificate::{Certificate, CertificateResult};
