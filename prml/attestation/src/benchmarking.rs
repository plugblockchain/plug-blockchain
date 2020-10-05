// Copyright 2019-2020 Plug New Zealand Limited
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Attestation benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};

use crate::Module as Attestation;

const SEED: u32 = 0;

benchmarks! {
	_{ }

    set_claim {
        let caller: T::AccountId = whitelisted_caller();
		let holder: T::AccountId = account("holder", 0, SEED);
		let topic = AttestationTopic::from(0xf00d);
		let value = AttestationValue::from(0xb33f);
    }: set_claim(RawOrigin::Signed(caller.clone()), holder.clone(), topic.clone(), value.clone())
    verify {
            let issuers: Vec<<T as frame_system::Trait>::AccountId> = vec![caller.clone()];
			assert_eq!(Attestation::<T>::issuers(holder.clone()), issuers);
			assert_eq!(Attestation::<T>::topics((holder.clone(), caller.clone())), [topic.clone()]);
			assert_eq!(Attestation::<T>::value((holder, caller, topic)), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn set_claim() {
        ExtBuilder::build().execute_with(|| {
            assert_ok!(test_benchmark_set_claim::<Test>());
        });
    }
}