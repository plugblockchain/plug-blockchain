// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Minimal fixed point arithmetic primitives and types for runtime.

#![cfg_attr(not(feature = "std"), no_std)]

/// Copied from `sp-runtime` and documented there.
#[macro_export]
macro_rules! assert_eq_error_rate {
	($x:expr, $y:expr, $error:expr $(,)?) => {
		assert!(
			($x) >= (($y) - ($error)) && ($x) <= (($y) + ($error)),
			"{:?} != {:?} (with error rate {:?})",
			$x,
			$y,
			$error,
		);
	};
}

pub mod biguint;
pub mod helpers_128bit;
pub mod traits;
mod per_things;
mod fixed_point;
mod rational128;

pub use fixed_point::{FixedPointNumber, FixedPointOperand, FixedI64, FixedI128, FixedU128};
pub use per_things::{PerThing, Percent, PerU16, Permill, Perbill, Perquintill};
pub use rational128::Rational128;

use sp_std::cmp::Ordering;

/// Trait for comparing two numbers with an threshold.
///
/// Returns:
/// - `Ordering::Greater` if `self` is greater than `other + threshold`.
/// - `Ordering::Less` if `self` is less than `other - threshold`.
/// - `Ordering::Equal` otherwise.
pub trait ThresholdOrd<T> {
	/// Compare if `self` is `threshold` greater or less than `other`.
	fn tcmp(&self, other: &T, epsilon: T) -> Ordering;
}

impl<T> ThresholdOrd<T> for T
where
	T: Ord + PartialOrd + Copy + Clone + traits::Zero + traits::Saturating,
{
	fn tcmp(&self, other: &T, threshold: T) -> Ordering {
		// early exit.
		if threshold.is_zero() {
			return self.cmp(&other)
		}

		let upper_bound = other.saturating_add(threshold);
		let lower_bound = other.saturating_sub(threshold);

		if upper_bound <= lower_bound {
			// defensive only. Can never happen.
			self.cmp(&other)
		} else {
			// upper_bound is guaranteed now to be bigger than lower.
			match (self.cmp(&lower_bound), self.cmp(&upper_bound)) {
				(Ordering::Greater, Ordering::Greater) => Ordering::Greater,
				(Ordering::Less, Ordering::Less) => Ordering::Less,
				_ => Ordering::Equal,
			}
		}

	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::Saturating;
	use sp_std::cmp::Ordering;

	#[test]
	fn epsilon_ord_works() {
		let b = 115u32;

		let e = Perbill::from_percent(10).mul_ceil(b);

		// [115 - 11,5 (103,5), 115 + 11,5 (126,5)] is all equal
		assert_eq!(103u32.tcmp(&b, e), Ordering::Equal);
		assert_eq!(104u32.tcmp(&b, e), Ordering::Equal);
		assert_eq!(115u32.tcmp(&b, e), Ordering::Equal);
		assert_eq!(120u32.tcmp(&b, e), Ordering::Equal);
		assert_eq!(126u32.tcmp(&b, e), Ordering::Equal);
		assert_eq!(127u32.tcmp(&b, e), Ordering::Equal);

		assert_eq!(128u32.tcmp(&b, e), Ordering::Greater);
		assert_eq!(102u32.tcmp(&b, e), Ordering::Less);
	}

	#[test]
	fn epsilon_ord_works_with_small_epc() {
		let b = 115u32;
		// way less than 1 percent. threshold will be zero. Result should be same as normal ord.
		let e = Perbill::from_parts(100) * b;

		// [115 - 11,5 (103,5), 115 + 11,5 (126,5)] is all equal
		assert_eq!(103u32.tcmp(&b, e), 103u32.cmp(&b));
		assert_eq!(104u32.tcmp(&b, e), 104u32.cmp(&b));
		assert_eq!(115u32.tcmp(&b, e), 115u32.cmp(&b));
		assert_eq!(120u32.tcmp(&b, e), 120u32.cmp(&b));
		assert_eq!(126u32.tcmp(&b, e), 126u32.cmp(&b));
		assert_eq!(127u32.tcmp(&b, e), 127u32.cmp(&b));

		assert_eq!(128u32.tcmp(&b, e), 128u32.cmp(&b));
		assert_eq!(102u32.tcmp(&b, e), 102u32.cmp(&b));
	}

	#[test]
	fn peru16_rational_does_not_overflow() {
		// A historical example that will panic only for per_thing type that are created with
		// maximum capacity of their type, e.g. PerU16.
		let _ = PerU16::from_rational_approximation(17424870u32, 17424870);
	}

	#[test]
	fn saturating_mul_works() {
		assert_eq!(Saturating::saturating_mul(2, i32::min_value()), i32::min_value());
		assert_eq!(Saturating::saturating_mul(2, i32::max_value()), i32::max_value());
	}
}
