use async_trait::async_trait;

use super::utils::{GateCommandTranslator, GateMessageHandler, EXCHANGE_NAME};
use crate::{
    clients::common_traits::{
        Candlestick, Level3OrderBook, OrderBook, OrderBookTopK, Ticker, Trade, BBO,
    },
    common::{command_translator::CommandTranslator, ws_client_internal::WSClientInternal},
    WSClient,
};
use nonzero_ext::nonzero;
use std::{num::NonZeroU32};

const WEBSOCKET_URL: &str = "wss://api.gateio.ws/ws/v4/";

/// The WebSocket client for Gate spot market.
///
/// * WebSocket API doc: <https://www.gate.io/docs/apiv4/ws/en/index.html>
/// * Trading at <https://www.gate.io/en/trade/BTC_USDT>

const UPLINK_LIMIT: (NonZeroU32, std::time::Duration) =
    (nonzero!(10u32), std::time::Duration::from_secs(1));


pub struct GateSpotWSClient {
    client: WSClientInternal<GateMessageHandler<'S'>>,
    translator: GateCommandTranslator<'S'>,
}

impl GateSpotWSClient {
    pub async fn new(tx: std::sync::mpsc::Sender<String>, url: Option<&str>) -> Self {
        let real_url = match url {
            Some(endpoint) => endpoint,
            None => WEBSOCKET_URL,
        };
        GateSpotWSClient {
            client: WSClientInternal::connect(
                EXCHANGE_NAME,
                real_url,
                GateMessageHandler {},
                Some(UPLINK_LIMIT),
                tx,
            )
            .await,
            translator: GateCommandTranslator {},
        }
    }
}

impl_trait!(Trade, GateSpotWSClient, subscribe_trade, "trades");
#[rustfmt::skip]
impl_trait!(OrderBook, GateSpotWSClient, subscribe_orderbook, "order_book_update");
#[rustfmt::skip]
impl_trait!(OrderBookTopK, GateSpotWSClient, subscribe_orderbook_topk, "order_book");
impl_trait!(BBO, GateSpotWSClient, subscribe_bbo, "book_ticker");
impl_trait!(Ticker, GateSpotWSClient, subscribe_ticker, "tickers");

impl_candlestick!(GateSpotWSClient);

panic_l3_orderbook!(GateSpotWSClient);

impl_ws_client_trait!(GateSpotWSClient);
