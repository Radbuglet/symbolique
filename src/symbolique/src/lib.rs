#[doc(hidden)]
pub mod macro_internals {
    pub use {
        lasso::{Spur, ThreadedRodeo},
        std::{
            clone::Clone,
            cmp::{Eq, Ord, PartialEq, PartialOrd},
            fmt,
            hash::Hash,
            marker::Copy,
            mem::transmute,
            sync::OnceLock,
        },
        symbolique_proc::__handle_magic,
    };
}

#[macro_export]
macro_rules! define_table {
    (
		$(#[$attr:meta])*
		$vis:vis struct $symbol:ident {
			$($name:expr $(,)?)*
		}
	) => {
		#[derive(
			$crate::macro_internals::Copy,
			$crate::macro_internals::Clone,
			$crate::macro_internals::Hash,
			$crate::macro_internals::Eq,
			$crate::macro_internals::PartialEq,
			$crate::macro_internals::Ord,
			$crate::macro_internals::PartialOrd,
		)]
		$(#[$attr])*
		#[repr(transparent)]
		$vis struct $symbol(pub $crate::macro_internals::Spur);

		impl $crate::macro_internals::fmt::Debug for $symbol {
			fn fmt(&self, f: &mut $crate::macro_internals::fmt::Formatter<'_>) -> $crate::macro_internals::fmt::Result {
				$crate::macro_internals::fmt::Debug::fmt(Self::rodeo().resolve(&self.0), f)
			}
		}

		impl $crate::macro_internals::fmt::Display for $symbol {
			fn fmt(&self, f: &mut $crate::macro_internals::fmt::Formatter<'_>) -> $crate::macro_internals::fmt::Result {
				f.write_str(Self::rodeo().resolve(&self.0))
			}
		}

		#[allow(dead_code)]
		impl $symbol {
			$crate::macro_internals::__handle_magic! {
				$(
					#[allow(non_upper_case_globals)]
					pub const __SYMBOLIQUE_MAGIC_HASH $name: $symbol = unsafe {
						// Safety: u32 -> NonZeroU32 -> Spur -> $symbol
						$crate::macro_internals::transmute(__SYMBOLIQUE_GEN_NZ_INCREMENTAL_ID)
					};
				)*
			}

			pub fn new(str: &str) -> Self {
				Self(Self::rodeo().get_or_intern(str))
			}

			pub fn as_str(&self) -> &str {
				Self::rodeo().resolve(&self.0)
			}

			pub fn rodeo() -> &'static $crate::macro_internals::ThreadedRodeo {
				static RODEO: $crate::macro_internals::OnceLock<$crate::macro_internals::ThreadedRodeo> =
					$crate::macro_internals::OnceLock::new();

				RODEO.get_or_init(|| {
					let rodeo = $crate::macro_internals::ThreadedRodeo::new();
					$(
						// N.B. duplicates are already checked by the compiler because we hash each
						// name to define a symbol for it.
						rodeo.get_or_intern($name);
					)*

					rodeo
				})
			}
		}

		#[allow(unused_macros)]
		macro_rules! $symbol {
			($name2:expr) => {
				$crate::macro_internals::__handle_magic! { $symbol:: __SYMBOLIQUE_MAGIC_HASH $name2 }
			};
		}
	};
}
