/// Constructs an Fee enum for a runtime. This is usually called automatically by the
/// `construct_runtime!` macro
#[macro_export]
macro_rules! impl_outer_fee {
	(
		$(#[$attr:meta])*
		pub enum $name:ident {
			$( $modules:tt , )*
		}
	) => {
		// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
		#[derive(Clone, PartialEq, Eq, $crate::codec::Encode, $crate::codec::Decode)]
		#[cfg_attr(feature = "std", derive(Debug, $crate::Serialize, $crate::Deserialize))]
		$(#[$attr])*
		#[allow(non_camel_case_types)]
		pub enum $name {
			$(
				$modules( $modules::Fee ),
			)*
		}
		// TODO: Generate Fee metadata #LUN-410
	}
}

/// Implement the `Fee` for a module.
///
/// ```rust
/// decl_fee!(
///    pub enum Fee {
///        Gst,
///        UsedStorage,
///    }
/// );
///# fn main() {}
/// ```
#[macro_export]
macro_rules! decl_fee {
	(
		$(#[$attr:meta])*
		pub enum Fee {
			$(
				$fees:tt
			)*
		}
	) => {
		// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
		#[derive(Clone, PartialEq, Eq, $crate::codec::Encode, $crate::codec::Decode)]
		#[cfg_attr(feature = "std", derive(Debug, $crate::Serialize, $crate::Deserialize))]
		/// Declared fees for this module.
		///
		$(#[$attr])*
		pub enum Fee {
			$(
				$fees
			)*
		}
		impl From<Fee> for () {
			fn from(_: Fee) -> () { () }
		}
		// TODO: Generate Fee metadata #LUN-410
	}
}