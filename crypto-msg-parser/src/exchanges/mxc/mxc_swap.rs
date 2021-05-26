use crypto_market_type::MarketType;

use super::super::utils::calc_quantity_and_volume;
use crate::{MessageType, TradeMsg, TradeSide};

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;

const EXCHANGE_NAME: &str = "mxc";

// https://mxcdevelop.github.io/APIDoc/contract.api.cn.html#4483df6e28
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawTradeMsg {
    p: f64, // price
    v: f64, // quantity
    T: i64, // 1, buy; 2, sell
    t: i64, // timestamp
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    channel: String,
    symbol: String,
    ts: i64,
    data: T,
}

pub(super) fn parse_trade(market_type: MarketType, msg: &str) -> Result<Vec<TradeMsg>> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawTradeMsg>>(msg)?;
    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();
    let raw_trade = ws_msg.data;

    let (quantity, volume) =
        calc_quantity_and_volume(EXCHANGE_NAME, market_type, &pair, raw_trade.p, raw_trade.v);

    let trade = TradeMsg {
        exchange: EXCHANGE_NAME.to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair,
        msg_type: MessageType::Trade,
        timestamp: raw_trade.t,
        price: raw_trade.p,
        quantity,
        volume,
        side: if raw_trade.T == 2 {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        },
        trade_id: raw_trade.t.to_string(),
        raw: serde_json::to_value(&raw_trade).unwrap(),
    };

    Ok(vec![trade])
}
