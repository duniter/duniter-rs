//  Copyright (C) 2019  Éloïs SANCHEZ
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Mocks for projects use dubp-block-doc

use dubp_currency_params::CurrencyParameters;

/// Generate mock currency parameters
pub fn gen_mock_currency_parameters() -> CurrencyParameters {
    CurrencyParameters {
        protocol_version: 10,
        c: 0.004,               // UD target growth rate (see Relative Theorie of Money)
        dt: 1000,               // Duration between the creation of two UD (in seconds)
        ud0: 10,                // Amount of the initial UD
        sig_period: 10000, // Minimum duration between the writing of 2 certifications from the same issuer (in seconds)
        sig_renew_period: 1000, // Minimum duration between two renewals of the same certification
        sig_stock: 100, // Maximum number of active certifications at the same time (for the same issuer)
        sig_window: 100,
        sig_validity: 100,
        sig_qty: 100,
        idty_window: 100,
        ms_window: 100,
        tx_window: 100,
        x_percent: 0.8,
        ms_validity: 100,
        ms_period: 100,
        step_max: 100,
        median_time_blocks: 100,
        avg_gen_time: 100,
        dt_diff_eval: 100,
        percent_rot: 0.5,
        ud_time0: 100,
        ud_reeval_time0: 100,
        dt_reeval: 100,
        fork_window_size: 100,
    }
}
