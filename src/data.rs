use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod dsp;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub output_item: String,
    pub output_item_count: f64,
    pub min_output_item_count: Option<f64>,
    pub max_output_item_count: Option<f64>,
    pub facility: String,
    pub time: f64,
    pub materials: Materials,
    pub image: Option<String>,
    pub market_data: Option<MarketData>,
}

impl Recipe {
    pub fn new() -> Self {
        Self {
            output_item: "".to_string(),
            output_item_count: 0.0,
            min_output_item_count: None,
            max_output_item_count: None,
            facility: "".to_string(),
            time: 0.0,
            materials: HashMap::new(),
            image: None,
            market_data: None,
        }
    }
}

pub type Materials = HashMap<String, f64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub last_update_attempt: i64,
    pub last_updated: i64,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub total_trade_count: Option<f64>,
    pub name: Option<String>,
}
