pub use anyhow;
pub use bevy;
pub use crossbeam;
pub use derive_more;
pub use rand;
pub use rand_pcg;
pub use serde;
pub use serde_json;
pub use tracing;

/// Temporary fix for bevy's derive macros since some of them rely on
/// `crate::X` items existing when using a re-exported bevy
#[macro_export]
#[rustfmt::skip] // Breaks use statement
macro_rules! fix_bevy_derive {
    ($bevy_path:path) => {
        pub(crate) use $bevy_path::{
            reflect::*
        };
    };
}
