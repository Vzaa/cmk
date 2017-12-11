#[macro_use]
extern crate clap;
extern crate serde_json;

extern crate cmk;

use std::fs::File;
use std::io::BufReader;
use std::process::exit;

use cmk::{Entry, Values};

const DEFFORMAT: &str = "%n(%a): $%i -> $%s (%d)\n=> %1 %2 %7";

fn main() {
    let matches = clap_app!(cmk =>
        (version: "0.1")
        (@arg PROXY: -p --proxy +takes_value "Proxy URL")
        (@arg LIMIT: -l --limit +takes_value "Queried currency limit (Default: 150)")
        (@arg FORMAT: -f --format +takes_value "Custom format")
        (@arg SUMMARY: -s --summary "Summary only")
        (@arg FILE: +required "Portfolio JSON File")
    ).get_matches();

    let summary = matches.is_present("SUMMARY");
    let json_path = matches.value_of("FILE").unwrap();
    let proxy: Option<&str> = matches.value_of("PROXY");
    let format_str: &str = matches.value_of("FORMAT").unwrap_or(DEFFORMAT);

    let limit: u32 = matches
        .value_of("LIMIT")
        .unwrap_or("150")
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("Invalid limit Values: {}", e);
            exit(1)
        });

    let coins = cmk::fetch_coin_list_data(proxy, limit).unwrap();

    let p: Vec<Entry> = File::open(json_path)
        .map(|f| serde_json::from_reader(BufReader::new(f)).unwrap())
        .unwrap();

    let Values(sum_usd, sum_init, m1, m24, m7) = p.iter()
        .map(|e| {
            let c = coins.get(&e.id).unwrap();
            e.values(&c)
        })
        .sum();

    if !summary {
        for e in p {
            let c = coins.get(&e.id).unwrap();
            let Values(val, init, c1, c24, c7) = e.values(&c);
            let out = format_str
                .to_owned()
                .replace("%n", &format!("{}", c.name))
                .replace("%a", &format!("{:.2}", e.amount))
                .replace("%i", &format!("{:.2}", init))
                .replace("%s", &format!("{:.2}", val))
                .replace("%d", &format!("{:.2}", val - init))
                .replace("%1", &format!("{:.2}", c1))
                .replace("%2", &format!("{:.2}", c24))
                .replace("%7", &format!("{:.2}", c7));

            println!("{}\n", out);
        }
    }

    let out = format_str
        .to_owned()
        .replace("%n", &format!("{}", "Total"))
        .replace("%i", &format!("{:.2}", sum_init))
        .replace("%a", "")
        .replace("%s", &format!("{:.2}", sum_usd))
        .replace("%d", &format!("{:.2}", sum_usd - sum_init))
        .replace("%1", &format!("{:.2}", m1))
        .replace("%2", &format!("{:.2}", m24))
        .replace("%7", &format!("{:.2}", m7));

    println!("{}", out);
}
