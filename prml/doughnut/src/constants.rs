// Copyright 2020 Plug New Zealand Limited
// This file is part of Plug.

// Plug is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Plug is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Plug. If not, see <http://www.gnu.org/licenses/>.

//! Plug Doughnut Constants

pub mod error_code {
	//! Plug Doughnut Error Code Constants
	pub const VERIFY_INVALID: u8 = 170;
	pub const VERIFY_UNSUPPORTED_VERSION: u8 = 171;
	pub const VERIFY_BAD_SIGNATURE_FORMAT: u8 = 172;
	pub const VERIFY_BAD_PUBLIC_KEY_FORMAT: u8 = 173;
	pub const VALIDATION_HOLDER_SIGNER_IDENTITY_MISMATCH: u8 = 180;
	pub const VALIDATION_EXPIRED: u8 = 181;
	pub const VALIDATION_PREMATURE: u8 = 182;
	pub const VALIDATION_CONVERSION: u8 = 183;
}