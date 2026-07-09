use zenoh::Config;

pub struct SessionConfig {
    pub config_file: Option<String>,
    pub mode: String,
    pub endpoints: String,
    pub listen: String,
    pub name: String,
    pub admin_enabled: bool,
    pub rest_port: String,
    pub multicast_scouting: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            config_file: None,
            mode: "peer".into(),
            endpoints: String::new(),
            listen: String::new(),
            name: "zsak-gui".into(),
            admin_enabled: true,
            rest_port: String::new(),
            multicast_scouting: true,
        }
    }
}

pub fn build_config(sc: &SessionConfig) -> Config {
    let mut config = match &sc.config_file {
        Some(f) => zenoh::Config::from_file(f).expect("Unable to open config file"),
        None => zenoh::Config::default(),
    };

    // Required options (mirrors main.rs set_required_options)
    config.insert_json5("plugins_loading/enabled", "true").unwrap();
    config.insert_json5("plugins/storage_manager/__required__", "true").unwrap();
    config.insert_json5(
        "metadata",
        &format!("{{ name: \"{}\" }}", sc.name),
    ).unwrap();
    config.insert_json5(
        "timestamping",
        r#"{ enabled: { router: true, peer: true, client: true }, drop_future_timestamp: false }"#,
    ).unwrap();
    config.insert_json5("transport/unicast/max_links", "10").unwrap();
    config.insert_json5("transport/link/tx/keep_alive", "2").unwrap();

    let mode = match sc.mode.as_str() {
        "client" => "client",
        "router" => "router",
        _ => "peer",
    };
    config.insert_json5("mode", &format!("\"{}\"", mode)).unwrap();

    if !sc.endpoints.is_empty() {
        config.insert_json5("connect/endpoints", &sc.endpoints).unwrap();
    }
    if !sc.listen.is_empty() {
        config.insert_json5("listen/endpoints", &sc.listen).unwrap();
    }
    if !sc.multicast_scouting {
        config.insert_json5("scouting/multicast/enabled", "false").unwrap();
    }
    if sc.admin_enabled {
        config.insert_json5("adminspace/enabled", "true").unwrap();
        config.insert_json5("adminspace/permissions", "{ read: true, write: true }").unwrap();
    }
    if !sc.rest_port.is_empty() {
        config.insert_json5("plugins/rest", &format!("{{http_port: {} }}", sc.rest_port)).unwrap();
    }

    config
}
