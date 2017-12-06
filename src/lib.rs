extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::str::FromStr;
use std::iter::Sum;

use chrono::{DateTime, TimeZone, Utc};
use serde::de::{Deserialize, Deserializer};
use serde::de;

#[derive(Deserialize, Debug)]
pub struct Coin {
    pub id: String,
    pub name: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub rank: u64,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub price_usd: f64,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub price_btc: f64,
    #[serde(rename = "24h_volume_usd", deserialize_with = "deserialize_fromstr")]
    pub t24h_volume_usd: f64,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub market_cap_usd: f64,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub available_supply: f64,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub total_supply: f64,
    #[serde(deserialize_with = "deserialize_fromstr_opt")]
    pub max_supply: Option<String>,
    #[serde(deserialize_with = "deserialize_fromstr_opt")]
    pub percent_change_1h: Option<f64>,
    #[serde(deserialize_with = "deserialize_fromstr_opt")]
    pub percent_change_24h: Option<f64>,
    #[serde(deserialize_with = "deserialize_fromstr_opt")]
    pub percent_change_7d: Option<f64>,
    #[serde(deserialize_with = "deserialize_utc")]
    pub last_updated: DateTime<Utc>,
}

fn deserialize_utc<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
    let s: String = Deserialize::deserialize(d)?;
    let t = s.parse().map_err(|_| de::Error::custom("Parse error"))?;
    Ok(Utc.timestamp(t, 0))
}

fn deserialize_fromstr<'de, D: Deserializer<'de>, T>(d: D) -> Result<T, D::Error>
where
    T: FromStr,
{
    let s: String = Deserialize::deserialize(d)?;
    let t = s.parse().map_err(|_| de::Error::custom("Parse error"))?;
    Ok(t)
}

fn deserialize_fromstr_opt<'de, D: Deserializer<'de>, T>(d: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
{
    let os: Option<String> = Deserialize::deserialize(d)?;

    if let Some(s) = os {
        let t = s.parse().map_err(|_| de::Error::custom("Parse error"))?;
        Ok(Some(t))
    } else {
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub id: String,
    pub amount: f64,
    pub init_cost: f64,
}

impl Entry {
    pub fn changes(&self, c: &Coin) -> Changes4 {
        let val = c.price_usd * self.amount;

        Changes4(
            val,
            self.init_cost,
            (val * c.percent_change_1h.unwrap_or(0.0)) / 100.0,
            (val * c.percent_change_24h.unwrap_or(0.0)) / 100.0,
            (val * c.percent_change_7d.unwrap_or(0.0)) / 100.0,
        )
    }
}

pub struct Changes4(pub f64, pub f64, pub f64, pub f64, pub f64);

impl Sum for Changes4 {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(
            Changes4(0.0, 0.0, 0.0, 0.0, 0.0),
            |Changes4(a1, a2, a3, a4, a5), Changes4(b1, b2, b3, b4, b5)| {
                Changes4(a1 + b1, a2 + b2, a3 + b3, a4 + b4, a5 + b5)
            },
        )
    }
}

pub fn fetch_coin_data(proxy: Option<String>, l: i32) -> Result<HashMap<String, Coin>, &'static str> {
    let client = if let Some(proxy_url) = proxy {
        reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(&proxy_url).map_err(|_| "Proxy error")?)
            .build()
            .map_err(|_| "Build error")?
    } else {
        reqwest::Client::builder()
            .build()
            .map_err(|_| "Build error")?
    };

    let resp = client
        .get(&format!("https://api.coinmarketcap.com/v1/ticker/?limit={}", l))
        .send()
        .map_err(|_| "Request send error")?;

    let c = serde_json::from_reader::<_, Vec<Coin>>(resp)
        .map_err(|_| "JSON parse error")?
        .into_iter()
        .map(|c| (c.symbol.clone(), c))
        .collect();

    Ok(c)
}
