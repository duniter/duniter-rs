use constants::*;
use duniter_documents::blockchain::v10::documents::block::{BlockV10Parameters, CurrencyName};
use *;

#[derive(Debug, Copy, Clone)]
pub struct CurrencyParameters {
    pub protocol_version: usize,
    pub c: f64,
    pub dt: u64,
    pub ud0: usize,
    pub sig_period: u64,
    pub sig_renew_period: u64,
    pub sig_stock: usize,
    pub sig_window: u64,
    pub sig_validity: u64,
    pub sig_qty: usize,
    pub idty_window: u64,
    pub ms_window: u64,
    pub tx_window: u64,
    pub x_percent: f64,
    pub ms_validity: u64,
    pub ms_period: u64,
    pub step_max: usize,
    pub median_time_blocks: usize,
    pub avg_gen_time: u64,
    pub dt_diff_eval: usize,
    pub percent_rot: f64,
    pub ud_time0: u64,
    pub ud_reeval_time0: u64,
    pub dt_reeval: u64,
}

impl From<(CurrencyName, BlockV10Parameters)> for CurrencyParameters {
    fn from(source: (CurrencyName, BlockV10Parameters)) -> CurrencyParameters {
        let (currency_name, block_params) = source;
        let sig_renew_period = match currency_name.0.as_str() {
            "default_currency" => *DEFAULT_SIG_RENEW_PERIOD,
            "g1" => 5_259_600,
            "g1-test" => 5_259_600 / 5,
            _ => *DEFAULT_SIG_RENEW_PERIOD,
        };
        let ms_period = match currency_name.0.as_str() {
            "default_currency" => *DEFAULT_MS_PERIOD,
            "g1" => 5_259_600,
            "g1-test" => 5_259_600 / 5,
            _ => *DEFAULT_MS_PERIOD,
        };
        let tx_window = match currency_name.0.as_str() {
            "default_currency" => *DEFAULT_TX_WINDOW,
            "g1" => 604_800,
            "g1-test" => 604_800,
            _ => *DEFAULT_TX_WINDOW,
        };
        CurrencyParameters {
            protocol_version: 10,
            c: block_params.c,
            dt: block_params.dt,
            ud0: block_params.ud0,
            sig_period: block_params.sig_period,
            sig_renew_period,
            sig_stock: block_params.sig_stock,
            sig_window: block_params.sig_window,
            sig_validity: block_params.sig_validity,
            sig_qty: block_params.sig_qty,
            idty_window: block_params.idty_window,
            ms_window: block_params.ms_window,
            tx_window,
            x_percent: block_params.x_percent,
            ms_validity: block_params.ms_validity,
            ms_period,
            step_max: block_params.step_max,
            median_time_blocks: block_params.median_time_blocks,
            avg_gen_time: block_params.avg_gen_time,
            dt_diff_eval: block_params.dt_diff_eval,
            percent_rot: block_params.percent_rot,
            ud_time0: block_params.ud_time0,
            ud_reeval_time0: block_params.ud_reeval_time0,
            dt_reeval: block_params.dt_reeval,
        }
    }
}

impl Default for CurrencyParameters {
    fn default() -> CurrencyParameters {
        CurrencyParameters::from((
            CurrencyName(String::from("default_currency")),
            BlockV10Parameters::default(),
        ))
    }
}

impl CurrencyParameters {
    /// Get max value of connectivity (=1/x_percent)
    pub fn max_connectivity(&self) -> f64 {
        1.0 / self.x_percent
    }
}

/// Get currency parameters
pub fn get_currency_params(
    blockchain_db: &BinFileDB<LocalBlockchainV10Datas>,
) -> Result<Option<CurrencyParameters>, DALError> {
    Ok(blockchain_db.read(|db| {
        if let Some(genesis_block) = db.get(&BlockId(0)) {
            if genesis_block.block.parameters.is_some() {
                Some(CurrencyParameters::from((
                    genesis_block.block.currency.clone(),
                    genesis_block.block.parameters.expect("safe unwrap"),
                )))
            } else {
                panic!("The genesis block are None parameters !");
            }
        } else {
            None
        }
    })?)
}
