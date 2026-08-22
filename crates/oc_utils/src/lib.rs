#[cfg(feature = "bevy")]
pub mod bevy;
pub mod collections;
pub mod d2;
#[cfg(feature = "debug")]
pub mod debug;
pub mod error;
pub mod image;
pub mod number;
pub mod random;
#[cfg(feature = "tiled")]
pub mod tileset;

#[macro_export]
macro_rules! let_some {
    ($pat:pat = $expr:expr, $or:expr) => {
        let Some($pat) = $expr else { $or };
    };
}

#[macro_export]
macro_rules! let_ok {
    ($pat:pat = $expr:expr, $or:expr) => {
        let Ok($pat) = $expr else { $or };
    };
}

#[macro_export]
macro_rules! return_if {
    ($expr:expr) => {
        if $expr {
            return;
        }
    };
    ($expr:expr, $return: expr) => {
        if $expr {
            $return;
        }
    };
}
