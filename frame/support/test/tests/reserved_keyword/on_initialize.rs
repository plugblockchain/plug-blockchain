macro_rules! reserved {
	($($reserved:ident)*) => {
		$(
			mod $reserved {
				use frame_support::additional_traits::MaybeDoughnutRef;
				pub use frame_support::dispatch;

				// `decl_module` expansion has added doughnut logic which requires system trait is implemented
				pub trait Trait: system::Trait {
					type Origin: MaybeDoughnutRef<Doughnut=()>;
					type BlockNumber: Into<u32>;
				}

				pub mod system {
					use sp_runtime::traits::PlugDoughnutApi;
					use frame_support::additional_traits::DelegatedDispatchVerifier;
					use frame_support::dispatch;

					pub trait Trait {
						type Doughnut: PlugDoughnutApi;
						type DelegatedDispatchVerifier: DelegatedDispatchVerifier<Doughnut = ()>;
					}

					pub fn ensure_root<R>(_: R) -> dispatch::DispatchResult {
						Ok(())
					}
				}

				frame_support::decl_module! {
					pub struct Module<T: Trait> for enum Call where origin: T::Origin, system=self {
						#[weight = 0]
						fn $reserved(_origin) -> dispatch::DispatchResult { unreachable!() }
					}
				}
			}
		)*
	}
}

reserved!(on_finalize on_initialize on_runtime_upgrade offchain_worker deposit_event);

fn main() {}
