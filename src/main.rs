use clap::ArgMatches;


mod action;
mod parser;



#[tokio::main]
async fn main() {
    env_logger::init();
    let matches = parser::arg_parser().get_matches();

    match matches.subcommand() {
        Some(("doctor", _)) => {
            action::do_doctor().await;
        },
        _ => { }
    }


    let mut config = match matches.get_one::<String>("config") {
        Some(fname) => zenoh::Config::from_file(fname).expect("Unable to open the Zenoh Config"),
        None => {
            zenoh::Config::default()
        }
    };

    set_required_options(&mut config);
    let mode = parse_top_level_args(&mut config, &matches);

    let z = zenoh::open(config.clone())
        .await
        .expect("Unable to open the Zenoh Session");

    let wait_for_ctrl_c = match matches.subcommand() {
        Some(("scout", sub_matches)) => {
            action::do_scout(&z, sub_matches).await;
            false
        }
        Some(("publish", sub_matches)) => {
            action::do_publish(&z, sub_matches).await;
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
        },
        Some(("queryable", sub_matches)) => {
            println!("Ctrl-C to quit");
            action::do_queryable(&z, sub_matches).await;
            false
        },
        Some(("stream", sub_matches)) => {
          if cfg!(feature = "video") {} else { }
          true
        },
        Some(("storage", sub_matches)) => {
            
            let complete = *sub_matches.get_one::<bool>("complete").unwrap();
            let kexpr = sub_matches.get_one::<String>("KEY_EXPR").unwrap();
            // @TODO: Eventually we should add additional params to configure the alignement algo.
            let replication = if sub_matches.get_one::<bool>("align").is_some() {
                ", replication: { interval: 3, sub_intervals: 5, hot: 6, warm: 24, propagation_delay: 10}"
            } else { "" };

            let storage_cfg = format!("{{ key_expr: \"{}\", volume: \"memory\",  complete: \"{}\" {}, }}", kexpr, complete, replication);

            if let Some(path) = std::env::var_os("ZSAK_HOME") {
                let config_path = 
                    path.into_string().unwrap() + "/config/config.json5";
                
                let cfg_template=
                    tokio::fs::read_to_string(config_path.clone())
                        .await.expect(&config_path);

                let cfg =
                    cfg_template.replace("$STORAGE", &storage_cfg);

                tokio::task::spawn(
                    tokio::process::Command::new("zenohd")
                        .args(["--inline-config", cfg.as_str()])
                        .output());
                true
            } else {
                panic!("ZSAK_HOME environment variable not set, please see README.md");
            }

        },
        _ => { false }
    };
    if wait_for_ctrl_c {
        println!("Ctrl-C to quit");
        tokio::signal::ctrl_c().await.unwrap();
    }
}

// --- Arg Parsing and Config Updating functions
fn set_required_options(config: &mut zenoh::config::Config) {
    config.insert_json5("plugins_loading/enabled", "true").unwrap();
    config.insert_json5("plugins/storage_manager/__required__","true").unwrap();
    config.insert_json5("metadata", r#"{ name: "Zenoh Swiss Army Knife", location: "My Laptop" }"#).unwrap();
    config.insert_json5("timestamping", r#"{ enabled: { router: true, peer: true, client: true }, drop_future_timestamp: false }"#).unwrap();
}

fn parse_top_level_args(config: &mut zenoh::config::Config, matches: &ArgMatches) -> String {
    if let Some(m) = matches.get_one::<String>("name") {
        config.insert_json5("metadata", &format!("{{ name: \"{}\" }}",m)).unwrap();
    }

    if let Some(ds) =  matches.get_one::<bool>("disable_scouting") {
        if *ds {
            println!("Scouting disabled");
            config.insert_json5("scouting/multicast/enabled", "false").unwrap();
        }
    }

    if let Some(es) = matches.get_one::<String>("endpoints") {
        config.insert_json5("connect/endpoints", es).unwrap();
    }

    if let Some(port) = matches.get_one::<String>("rest") {
        config.insert_json5(
            "plugins/rest",
            &format!("{{http_port: {} }}", port)).unwrap();
    }

    if let Some(admin) = matches.get_one::<bool>("admin") {
        if *admin {
            config.insert_json5("adminspace/enabled", "true").unwrap();
            config.insert_json5("adminspace/permissions", "{ read: true, write: true }").unwrap();
        }
    }

    let mode = match matches.get_one::<String>("mode") {
        Some(m) => {
            let mode = match m.as_str() {
                "peer" => {"peer"},
                "client" => { "client" },
                "router" => { "router" },
                _ => {
                    println!("Invalid mode \"{}\" defaulting to \"peer\"", m);
                    "peer"
                }

            };
            config.insert_json5("mode", &format!("\"{}\"",mode)).unwrap();
            mode
        },
        None => { "peer" }
    };

    mode.into()
}