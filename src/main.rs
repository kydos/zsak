use crate::action::do_list;
use crate::parser::resolve_argument;
use clap::ArgMatches;
use colored::Colorize;
use zenoh::config::WhatAmI;
use zenoh::liveliness::LivelinessToken;

mod action;
mod parser;

#[tokio::main]
async fn main() {
    env_logger::init();
    let matches = parser::arg_parser().get_matches();

    if let Some(("doctor", _)) = matches.subcommand() {
        action::do_doctor().await;
    }

    let mut config = match matches.get_one::<String>("config") {
        Some(fname) => zenoh::Config::from_file(fname).expect("Unable to open the Zenoh Config"),
        None => zenoh::Config::default(),
    };

    set_required_options(&mut config);
    parse_top_level_args(&mut config, &matches);
    let mut _token: Option<LivelinessToken> = None;
    let z = zenoh::open(config.clone())
        .await
        .expect("Unable to open the Zenoh Session");

    let wait_for_ctrl_c = match matches.subcommand() {
        Some(("scout", sub_matches)) => {
            let scout_interval = resolve_argument::<u64>(sub_matches, "SCOUT_INTERVAL", false)
                .await
                .expect("Scout interval should be an integer");

            let scouted = action::do_scout(&z, scout_interval).await;

            for (sn, (_, hello)) in scouted.into_iter().enumerate() {
                println!("{}({}):", "scouted".bold(), sn);
                println!("\t{}: {}", "Zenoh ID".bold(), hello.zid());
                println!("\t{}: {}", "Kind".bold(), hello.whatami());
                println!(
                    "\t{}\n:{}\n",
                    "Locators".bold(),
                    hello
                        .locators()
                        .iter()
                        .fold("".to_string(), |a, l| { a + &l.to_string() + ",\n\t   " })
                );
            }

            false
        }
        Some(("list", sub_matches)) => {
            let kind = if let Some(true) = sub_matches.get_one::<bool>("router") {
                WhatAmI::Router as usize
            } else if let Some(true) = sub_matches.get_one::<bool>("peer") {
                WhatAmI::Peer as usize
            } else if let Some(true) = sub_matches.get_one::<bool>("client") {
                WhatAmI::Client as usize
            } else {
                WhatAmI::Router as usize | WhatAmI::Peer as usize | WhatAmI::Client as usize
            };
            for (id, wai) in action::do_list(&z, kind).await {
                println!("- {} ({})", id.bold(), wai);
            }
            false
        }
        Some(("publish", sub_matches)) => {
            action::do_publish(&z, sub_matches).await;
            false
        }
        Some(("delete", sub_matches)) => {
            action::do_delete(&z, sub_matches).await;
            false
        }
        Some(("subscribe", sub_matches)) => {
            println!("Ctrl-C to quit");
            action::do_subscribe(&z, sub_matches).await;
            false
        }
        Some(("query", sub_matches)) => {
            action::do_query(&z, sub_matches).await;
            false
        }
        Some(("queryable", sub_matches)) => {
            println!("Ctrl-C to quit");
            action::do_queryable(&z, sub_matches).await;
            false
        }
        Some(("stream", _sub_matches)) => {
            if cfg!(feature = "video") {
                println!("Not Implemented Yet");
                false
            } else {
                println!("You need to enable the `video` feature");
                false
            }
        }
        Some(("storage", sub_matches)) => {
            let complete = *sub_matches.get_one::<bool>("complete").unwrap();
            let kexpr = sub_matches.get_one::<String>("KEY_EXPR").unwrap();
            // @TODO: Eventually we should add additional params to configure the alignement algo.
            let replication = if sub_matches.get_one::<bool>("align").is_some() {
                ", replication: { interval: 3, sub_intervals: 5, hot: 6, warm: 24, propagation_delay: 10}"
            } else {
                ""
            };

            let storage_cfg = format!(
                "{{ key_expr: \"{}\", volume: \"memory\",  complete: \"{}\" {}, }}",
                kexpr, complete, replication
            );

            if let Some(path) = std::env::var_os("ZSAK_HOME") {
                let config_path = path.into_string().unwrap() + "/config/config.json5";

                let cfg_template = tokio::fs::read_to_string(config_path.clone())
                    .await
                    .expect(&config_path);

                let cfg = cfg_template.replace("$STORAGE", &storage_cfg);

                tokio::task::spawn(
                    tokio::process::Command::new("zenohd")
                        .args(["--cfg", cfg.as_str()])
                        .output(),
                );
                true
            } else {
                panic!("ZSAK_HOME environment variable not set, please see README.md");
            }
        }
        Some(("liveliness", _sub_matches)) => {
            if let Some(key_expr) = _sub_matches.get_one::<String>("declare") {
                // If we drop the token, it loses liveliness.
                _token = Some(action::do_declare_liveliness_token(&z, key_expr).await);
                true
            } else if let Some(key_expr) = _sub_matches.get_one::<String>("subscribe") {
                action::do_subscribe_liveliness_token(&z, key_expr).await;
                false
            } else if let Some(key_expr) = _sub_matches.get_one::<String>("query") {
                action::do_query_liveliness(&z, key_expr).await;
                false
            } else {
                false
            }
        }
        Some(("graph", _sub_matches)) => {
            let zid = if let Some(zid) = _sub_matches.get_one::<String>("router") {
                zid.to_string()
            } else {
                let scouted = do_list(&z, WhatAmI::Router as usize).await;
                let (id, _) = scouted.first().expect("No Zenoh Router found");
                id.to_string()
            };
            let query = format!("@/{}/router/linkstate/routers", zid);
            // this is a single reply
            let replies = z.get(query).await.unwrap();
            let reply = replies.recv_async().await.unwrap();
            let result = reply.result().unwrap();
            let graph = result
                .payload()
                .try_to_string()
                .expect("Can't decode payload")
                .to_string();
            let split = graph.split('\n');
            for line in split {
                println!("{}", line);
            }
            false
        }
        _ => false,
    };
    if wait_for_ctrl_c {
        println!("Ctrl-C to quit");
        tokio::signal::ctrl_c().await.unwrap();
    }
}

// --- Arg Parsing and Config Updating functions
fn set_required_options(config: &mut zenoh::config::Config) {
    config
        .insert_json5("plugins_loading/enabled", "true")
        .unwrap();
    config
        .insert_json5("plugins/storage_manager/__required__", "true")
        .unwrap();
    config
        .insert_json5(
            "metadata",
            r#"{ name: "Zenoh Swiss Army Knife", location: "My Laptop" }"#,
        )
        .unwrap();
    config.insert_json5("timestamping",
                        r#"{ enabled: { router: true, peer: true, client: true }, drop_future_timestamp: false }"#).unwrap();
    config
        .insert_json5("transport/unicast/max_links", "10")
        .unwrap();
    config
        .insert_json5("transport/link/tx/keep_alive", "2")
        .unwrap();
}

fn parse_top_level_args(config: &mut zenoh::config::Config, matches: &ArgMatches) {
    let has_name = if let Some(m) = matches.get_one::<String>("name") {
        config
            .insert_json5("metadata", &format!("{{ name: \"{}\" }}", m))
            .unwrap();
        true
    } else {
        false
    };

    if let Some(ds) = matches.get_one::<bool>("no-multicast-scouting") {
        if *ds {
            println!("Scouting disabled");
            config
                .insert_json5("scouting/multicast/enabled", "false")
                .unwrap();
        }
    }

    if let Some(es) = matches.get_one::<String>("endpoints") {
        config.insert_json5("connect/endpoints", es).unwrap();
    }

    if let Some(port) = matches.get_one::<String>("rest") {
        config
            .insert_json5("plugins/rest", &format!("{{http_port: {} }}", port))
            .unwrap();
    }

    if let Some(admin) = matches.get_one::<bool>("admin") {
        if *admin || has_name {
            config.insert_json5("adminspace/enabled", "true").unwrap();
            config
                .insert_json5("adminspace/permissions", "{ read: true, write: true }")
                .unwrap();
        }
    }

    if let Some(m) = matches.get_one::<String>("mode") {
        let mode = match m.as_str() {
            "peer" => "peer",
            "client" => "client",
            "router" => "router",
            _ => {
                println!("Invalid mode \"{}\" defaulting to \"peer\"", m);
                "peer"
            }
        };
        config
            .insert_json5("mode", &format!("\"{}\"", mode))
            .unwrap();
    }
}
