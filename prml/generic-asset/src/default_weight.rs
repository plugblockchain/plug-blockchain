use frame_support::weights::{
	constants::{RocksDbWeight as DbWeight},
	Weight,
};

impl crate::WeightInfo for () {
	fn transfer() -> Weight {
		// taken from frame/balances
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn create() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn update_asset_info() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn mint() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn burn() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn create_reserved() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
	fn update_permission() -> Weight {
		(65_949_000 as Weight)
			.saturating_mul(84)
			.saturating_add(DbWeight::get().reads_writes(4, 2))
	}
}
