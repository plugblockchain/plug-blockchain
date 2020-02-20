// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Timestamp pallet benchmarking.

use super::*;

use sp_std::prelude::*;

use frame_system::RawOrigin;
use frame_benchmarking::benchmarks;
use sp_runtime::traits::Dispatchable;

/// Benchmark `set` extrinsic.
struct Set;
impl<T: Trait> BenchmarkingSetup<T, Call<T>, RawOrigin<T::AccountId, T::Doughnut>> for Set {
	fn components(&self) -> Vec<(BenchmarkParameter, u32, u32)> {
		vec![
			// Current time ("Now")
			(BenchmarkParameter::N, 1, 100),
		]
	}

	fn instance(&self, components: &[(BenchmarkParameter, u32)])
		-> Result<(Call<T>, RawOrigin<T::AccountId, T::Doughnut>), &'static str>
	{
		let user_origin = RawOrigin::None;
		let now = components.iter().find(|&c| c.0 == BenchmarkParameter::N).unwrap().1;

benchmarks! {
	_ {
		let n in 1 .. MAX_TIME => ();
	}

		let components = <SelectedBenchmark as BenchmarkingSetup<T, crate::Call<T>, RawOrigin<T::AccountId, T::Doughnut>>>::components(&selected_benchmark);
		let mut results: Vec<BenchmarkResults> = Vec::new();

		// Select the component we will be benchmarking. Each component will be benchmarked.
		for (name, low, high) in components.iter() {
			// Create up to `STEPS` steps for that component between high and low.
			let step_size = ((high - low) / steps).max(1);
			let num_of_steps = (high - low) / step_size;
			for s in 0..num_of_steps {
				// This is the value we will be testing for component `name`
				let component_value = low + step_size * s;

				// Select the mid value for all the other components.
				let c: Vec<(BenchmarkParameter, u32)> = components.iter()
					.map(|(n, l, h)|
						(*n, if n == name { component_value } else { (h - l) / 2 + l })
					).collect();

				// Run the benchmark `repeat` times.
				for _ in 0..repeat {
					// Set up the externalities environment for the setup we want to benchmark.
					let (call, caller) = <SelectedBenchmark as BenchmarkingSetup<
						T,
						Call<T>,
						RawOrigin<T::AccountId, T::Doughnut>,
					>>::instance(&selected_benchmark, &c)?;
					// Commit the externalities to the database, flushing the DB cache.
					// This will enable worst case scenario for reading from the database.
					benchmarking::commit_db();
					// Run the benchmark.
					let start = benchmarking::current_time();
					call.dispatch(caller.into())?;
					let finish = benchmarking::current_time();
					let elapsed = finish - start;
					results.push((c.clone(), elapsed));
					// Wipe the DB back to the genesis state.
					benchmarking::wipe_db();
				}
			}
		}

		return Ok(results);
	}
}
