#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::env;
use std::iter::Sum;

#[cfg(test)]
mod tests {
    use super::*;

    const SANDBOX_API_URL: &str = "https://sandbox-api.coinmarketcap.com/";

    #[test]
    fn test_map() {
        let json_txt = include_str!("../map_test.json");
        let list: Result<CryptocurrencyMap, _> = serde_json::from_str(&json_txt);
        assert!(list.is_ok());
    }

    #[test]
    fn test_live_map() {
        let handle = CmkHandle::new(SANDBOX_API_URL, "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c");
        let list = handle.fetch_map();
        assert!(list.is_ok());
    }

    #[test]
    fn test_quotes() {
        let json_txt = include_str!("../quotes_test.json");
        let quotes: Result<CryptocurrencyQuotes, _> = serde_json::from_str(&json_txt);
        assert!(quotes.is_ok());
    }

    #[test]
    fn test_live_quotes() {
        let handle = CmkHandle::new(SANDBOX_API_URL, "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c");
        let quotes = handle.fetch_quotes_by_slug(&["bitcoin", "dogecoin"]);
        assert!(quotes.is_ok());
    }
}

#[derive(Deserialize, Debug)]
pub struct CryptocurrencyMap {
    data: Vec<Id>,
    //status: Status,
}

#[derive(Deserialize, Debug)]
pub struct CryptocurrencyQuotes {
    data: HashMap<String, Cryptocurrency>,
    //status: Status,
}

impl CryptocurrencyQuotes {
    pub fn get_by_slug(&self, slug: &str) -> Option<&Cryptocurrency> {
        self.data.values().find(|c| c.slug == slug)
    }

    pub fn get_by_id(&self, id: &str) -> Option<&Cryptocurrency> {
        self.data.get(id)
    }
}

#[derive(Deserialize, Debug)]
pub struct Id {
    id: i32,
    name: String,
    symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    error_code: i32,
    error_message: String,
    elapsed: i32,
    credit_count: i32,
}

#[derive(Deserialize, Debug)]
pub struct Cryptocurrency {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub quote: HashMap<String, Quote>,
}

#[derive(Deserialize, Debug)]
pub struct Quote {
    pub price: f64,
    pub percent_change_1h: Option<f64>,
    pub percent_change_24h: Option<f64>,
    pub percent_change_7d: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub id: String,
    pub amount: f64,
    pub init_cost: f64,
}

impl Entry {
    pub fn values(&self, c: &Cryptocurrency) -> Values {
        let val = c.quote["USD"].price * self.amount;
        let c1 = c.quote["USD"].percent_change_1h.unwrap_or(0.0);
        let c2 = c.quote["USD"].percent_change_24h.unwrap_or(0.0);
        let c7 = c.quote["USD"].percent_change_7d.unwrap_or(0.0);

        Values(
            val,
            self.init_cost,
            (val / (c1 + 100.0)) * c1,
            (val / (c2 + 100.0)) * c2,
            (val / (c7 + 100.0)) * c7,
        )
    }
}

pub struct CmkHandle {
    api_url: String,
    api_key: String,
    proxy: Option<String>,
}

impl CmkHandle {
    pub fn new(api_url: &str, api_key: &str) -> CmkHandle {
        CmkHandle {
            api_url: api_url.into(),
            api_key: api_key.into(),
            proxy: None,
        }
    }

    pub fn set_proxy(&mut self, proxy: &str) {
        self.proxy = Some(proxy.into());
    }

    fn get_client(&self, url: &str) -> Result<ureq::Request, &'static str> {
        let mut client = ureq::get(url);

        client.set("X-CMC_PRO_API_KEY", &self.api_key);

        if let Some(proxy_url) = &self.proxy {
            let p = ureq::Proxy::new(&proxy_url).map_err(|_| "Proxy Error")?;
            client.set_proxy(p);
        } else if let Ok(proxy_url) = env::var("http_proxy") {
            let p = ureq::Proxy::new(&proxy_url).map_err(|_| "Proxy Error")?;
            client.set_proxy(p);
        };

        Ok(client)
    }

    pub fn fetch_map(&self) -> Result<CryptocurrencyMap, &'static str> {
        let resp = self
            .get_client(&format!("{}/{}", self.api_url, "/v1/cryptocurrency/map"))?
            .call()
            .into_string()
            .unwrap();

        serde_json::from_str::<CryptocurrencyMap>(&resp).map_err(|_| "JSON parse error")
    }

    pub fn fetch_quotes_by_slug(
        &self,
        slugs: &[&str],
    ) -> Result<CryptocurrencyQuotes, &'static str> {
        let slugs_txt = slugs.join(",");

        let resp = self
            .get_client(&format!(
                "{}/{}",
                self.api_url, "/v1/cryptocurrency/quotes/latest"
            ))?
            .query("slug", &slugs_txt)
            .call()
            .into_string()
            .unwrap();

        serde_json::from_str::<CryptocurrencyQuotes>(&resp).map_err(|_| "JSON parse error")
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
