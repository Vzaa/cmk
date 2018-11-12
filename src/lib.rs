extern crate chrono;
extern crate curl;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::str::FromStr;
use std::iter::Sum;
use std::env;

use chrono::{DateTime, TimeZone, Utc};
use serde::de::{Deserialize, Deserializer};
use serde::de;
use curl::easy::Easy;

const API_URL: &str = "https://api.coinmarketcap.com/v1/ticker/";

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
    pub fn values(&self, c: &Coin) -> Values {
        let val = c.price_usd * self.amount;
        let c1 = c.percent_change_1h.unwrap_or(0.0);
        let c2 = c.percent_change_24h.unwrap_or(0.0);
        let c7 = c.percent_change_7d.unwrap_or(0.0);

        Values(
            val,
            self.init_cost,
            ((val / (c1 + 100.0)) * c1),
            ((val / (c2 + 100.0)) * c2),
            ((val / (c7 + 100.0)) * c7),
        )
    }
}

pub struct Values(pub f64, pub f64, pub f64, pub f64, pub f64);

impl Sum for Values {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(
            Values(0.0, 0.0, 0.0, 0.0, 0.0),
            |Values(a1, a2, a3, a4, a5), Values(b1, b2, b3, b4, b5)| {
                Values(a1 + b1, a2 + b2, a3 + b3, a4 + b4, a5 + b5)
            },
        )
    }
}

fn get_client(proxy: Option<&str>) -> Result<Easy, &'static str> {
    let mut client = Easy::new();

    if let Some(proxy_url) = proxy {
        client.proxy(proxy_url).map_err(|_| "Proxy Error")?;
    } else if let Ok(proxy_url) = env::var("http_proxy") {
        client.proxy(&proxy_url).map_err(|_| "Proxy Error")?;
    };

    Ok(client)
}

pub fn fetch_coin_list(proxy: Option<&str>, l: u32) -> Result<HashMap<String, Coin>, &'static str> {
    let mut client = get_client(proxy)?;
    let mut resp: Vec<u8> = Vec::new();

    client.url(&format!("{}/?limit={}", API_URL, l)).unwrap();

    {
        let mut transfer = client.transfer();
        transfer
            .write_function(|data| {
                resp.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
        transfer.perform().unwrap();
    }

    let c = serde_json::from_slice::<Vec<Coin>>(&resp)
        .map_err(|_| "JSON parse error")?
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    Ok(c)
}

pub fn fetch_coin(proxy: Option<&str>, id: &str) -> Result<Coin, &'static str> {
    let mut client = get_client(proxy)?;
    let mut resp: Vec<u8> = Vec::new();

    client.url(&format!("{}/{}/", API_URL, id)).unwrap();

    {
        let mut transfer = client.transfer();
        transfer
            .write_function(|data| {
                resp.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
        transfer.perform().unwrap();
    }

    serde_json::from_slice::<Vec<Coin>>(&resp)
        .map_err(|_| "JSON parse error")?
        .pop()
        .ok_or("Emptry Response")
}
