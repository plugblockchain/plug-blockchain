// Copyright (C) 2019 Centrality Investments Limited
// This file is part of PLUG.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//!
//! The DispatchVerifier impl for this runtime permission domain
//!
use crate::Runtime;
use node_primitives::Doughnut;
use support::additional_traits::DispatchVerifier;

impl DispatchVerifier<Doughnut> for Runtime {
	const DOMAIN: &'static str = "plug";

	fn verify(
		_doughnut: &Doughnut,
		_module: &str,
		_method: &str,
	) -> Result<(), &'static str> {
		Err("Doughnut dispatch verification is not implemented for this domain")
	}
}
