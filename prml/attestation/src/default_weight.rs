use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
	fn set_claim() -> Weight {
		// taken from frame/balances
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn remove_claim() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
}
