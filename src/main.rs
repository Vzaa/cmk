#[macro_use]
extern crate clap;
extern crate serde_json;

extern crate cmk;

use std::fs::File;
use std::io::BufReader;
use std::process::exit;

use cmk::{Entry, Values};

fn main() {
    let matches = clap_app!(cmk =>
        (version: "0.1")
        (@arg PROXY: -p --proxy +takes_value "Proxy URL")
        (@arg LIMIT: -l --limit +takes_value "Queried currency limit (Default: 150)")
        (@arg FILE: +required "Portfolio JSON File")
    ).get_matches();

    let json_path = matches.value_of("FILE").unwrap();

    let limit: u32 = matches
        .value_of("JSON")
        .unwrap_or("150")
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("Invalid limit Values: {}", e);
            exit(1)
        });

    let proxy: Option<&str> = matches.value_of("PROXY");

    let coins = cmk::fetch_coin_data(proxy, limit).unwrap();

    let p: Vec<Entry> = File::open(json_path)
        .map(|f| serde_json::from_reader(BufReader::new(f)).unwrap())
        .unwrap();

    let Values(sum_usd, sum_init, m1, m24, m7) = p.iter()
        .map(|e| {
            let c = coins.get(&e.id).unwrap();
            e.values(&c)
        })
        .sum();

    for e in p {
        let c = coins.get(&e.id).unwrap();
        let Values(val, init, c1, c24, c7) = e.values(&c);
        eprintln!(
            "{}({:.2}): ${:.2} -> ${:.2} ({:.2}, {:.2}%)\n=> {:.2} {:.2} {:.2}\n",
            c.name,
            e.amount,
            init,
            val,
            val - init,
            ((val - init) / init) * 100.0,
            c1,
            c24,
            c7
        );
    }

    println!(
        "${:.2} -> ${:.2} ({:.2}, {:.2}%) => {:.2} {:.2} {:.2}",
        sum_init,
        sum_usd,
        sum_usd - sum_init,
        ((sum_usd - sum_init) / sum_init) * 100.0,
        m1,
        m24,
        m7
    );
}
