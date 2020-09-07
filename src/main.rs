use std::fs::File;
use std::io::BufReader;

use clap::clap_app;
use prettytable::{cell, row, table, Cell, Row, Table};

use cmk::{CmkHandle, Cryptocurrency, Entry, Values};

const DEFFORMAT: &str =
    "%n ($%u): %a\n $%i -> $%s (%d, %D%)\n 1h: %1 (%11%)\n 24h: %2 (%22%)\n 7d: %7 (%77%)";

fn formatted_str(c: Option<&Cryptocurrency>, e: Option<&Entry>, v: &Values, f: &str) -> String {
    let Values(val, init, c1, c24, c7) = *v;

    f.to_owned()
        .replace("\\n", "\n")
        .replace("%i", &format!("{:.2}", init))
        .replace("%s", &format!("{:.2}", val))
        .replace("%d", &format!("{:.2}", val - init))
        .replace("%D", &format!("{:.2}", 100.0 * (val - init) / init))
        .replace("%11", &format!("{:.2}", 100.0 * c1 / (val - c1)))
        .replace("%22", &format!("{:.2}", 100.0 * c24 / (val - c24)))
        .replace("%77", &format!("{:.2}", 100.0 * c7 / (val - c7)))
        .replace("%1", &format!("{:.2}", c1))
        .replace("%2", &format!("{:.2}", c24))
        .replace("%7", &format!("{:.2}", c7))
        .replace("%n", c.map(|x| x.name.as_str()).unwrap_or("Total"))
        .replace(
            "%u",
            &c.map(|x| format!("{}", x.quote["USD"].price))
                .unwrap_or_else(|| "N/A".to_owned()),
        )
        .replace(
            "%a",
            &e.map(|x| format!("{:.2}", x.amount))
                .unwrap_or_else(|| "N/A".to_owned()),
        )
}

fn fill_row(c: Option<&Cryptocurrency>, e: Option<&Entry>, v: &Values, t: &mut Table) {
    let Values(val, init, c1, c24, c7) = *v;
    let cel = |v, color, b, a| {
        let style = if color {
            if v > 0.0 {
                "Fg"
            } else {
                "Br"
            }
        } else {
            ""
        };

        Cell::new(&format!("{}{:.2}{}", b, v, a)).style_spec(style)
    };

    let nr = Row::new(vec![
        Cell::new(c.map(|x| x.symbol.as_str()).unwrap_or("Total")),
        Cell::new(
            &c.map(|x| format!("${}", x.quote["USD"].price))
                .unwrap_or_else(|| "N/A".to_owned()),
        ),
        Cell::new(
            &e.map(|x| format!("{:.2}", x.amount))
                .unwrap_or_else(|| "N/A".to_owned()),
        ),
        cel(init, false, "$", ""),
        cel(val, false, "$", ""),
        cel(val - init, true, "$", ""),
        cel(100.0 * (val - init) / init, true, "", "%"),
        cel(c1, true, "$", ""),
        cel(100.0 * c1 / (val - c1), true, "", "%"),
        cel(c24, true, "$", ""),
        cel(100.0 * c24 / (val - c24), true, "", "%"),
        cel(c7, true, "$", ""),
        cel(100.0 * c7 / (val - c7), true, "", "%"),
    ]);

    t.add_row(nr);
}

const DEF_API_URL: &str = "https://pro-api.coinmarketcap.com/";

fn main() {
    let matches = clap_app!(cmk =>
        (version: "0.1")
        (@arg PROXY: -p --proxy +takes_value "Proxy URL")
        (@arg API_URL: -u --url +takes_value "Custom API URL")
        (@arg FORMAT: -f --format +takes_value "Custom format")
        (@arg SUMMARY: -s --summary "Summary only")
        (@arg TABLE: -t --table "Print table")
        (@arg FILE: +required "Portfolio JSON File")
        (@arg API_KEY: +required "API Key")
    )
    .get_matches();

    let summary = matches.is_present("SUMMARY");
    let json_path = matches.value_of("FILE").unwrap();
    let proxy: Option<&str> = matches.value_of("PROXY");
    let format_str: &str = matches.value_of("FORMAT").unwrap_or(DEFFORMAT);
    let table = matches.is_present("TABLE");
    let api_key = matches.value_of("API_KEY").unwrap();
    let api_url = matches.value_of("API_URL").unwrap_or(DEF_API_URL);

    let mut user_list: Vec<Entry> = File::open(json_path)
        .map(|f| serde_json::from_reader(BufReader::new(f)).unwrap())
        .unwrap();

    let mut handle = CmkHandle::new(api_url, api_key);
    if let Some(proxy) = proxy {
        handle.set_proxy(proxy);
    }

    let slugs: Vec<&str> = user_list.iter().map(|c| c.id.as_str()).collect();

    let coins = handle.fetch_quotes_by_slug(&slugs).unwrap();

    user_list.sort_by(|a, b| {
        let c_a = &coins.get_by_slug(&a.id).unwrap();
        let c_b = &coins.get_by_slug(&b.id).unwrap();
        let Values(_val_a, _init_a, _c1_a, _c24_a, _c7_a) = a.values(c_a);
        let Values(_val_b, _init_b, _c1_b, _c24_b, _c7_b) = b.values(c_b);
        let _1h_a = c_a.quote["USD"].percent_change_1h.unwrap_or(0.0);
        let _1h_b = c_b.quote["USD"].percent_change_1h.unwrap_or(0.0);
        _1h_a.partial_cmp(&_1h_b).unwrap()
    });

    let v = user_list
        .iter()
        .map(|e| {
            let c = &coins.get_by_slug(&e.id).unwrap();
            e.values(c)
        })
        .sum();

    let mut t = table!([
        "Name", "Unit USD", "Owned", "Init", "Value", "Earned", "Earned %", "1h", "1h%", "24h",
        "24h%", "7d", "7d%"
    ]);

    if !summary {
        for e in user_list {
            let c = &coins.get_by_slug(&e.id).unwrap();
            let out = formatted_str(Some(c), Some(&e), &e.values(c), format_str);
            fill_row(Some(c), Some(&e), &e.values(c), &mut t);
            if !table {
                println!("{}\n", out);
            }
        }
    }

    let out = formatted_str(None, None, &v, format_str);
    fill_row(None, None, &v, &mut t);

    if table {
        t.printstd();
    } else {
        println!("{}", out);
    }
}
