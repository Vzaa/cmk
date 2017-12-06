extern crate cmk;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::BufReader;

use cmk::{Changes4, Entry};

fn main() {
    let json_path = env::args().nth(1).unwrap();
    let proxy = env::args().nth(2);

    let coins = cmk::fetch_coin_data(proxy, 150).unwrap();

    let p: Vec<Entry> = File::open(json_path)
        .map(|f| serde_json::from_reader(BufReader::new(f)).unwrap())
        .unwrap();

    let Changes4(sum_usd, sum_init, m1, m24, m7) = p.iter()
        .map(|e| {
            let c = coins.get(&e.id).unwrap();
            e.changes(&c)
        })
        .sum();

    for e in p {
        let c = coins.get(&e.id).unwrap();
        let Changes4(val, init, c1, c24, c7) = e.changes(&c);
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
