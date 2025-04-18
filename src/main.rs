mod action;
mod parser;

#[tokio::main]
async fn main() {
    let matches = parser::arg_parser().get_matches();

    let config = match matches.get_one::<String>("config") {
        Some(fname) => zenoh::Config::from_file(fname).expect("Unable to open the Zenoh Config"),
        None => zenoh::Config::default(),
    };

    let z = zenoh::open(config.clone())
        .await
        .expect("Unable to open the Zenoh Session");

    match matches.subcommand() {
        Some(("scout", sub_matches)) => {
            action::do_scout(&z, sub_matches).await;
        }
        Some(("publish", sub_matches)) => {
            action::do_publish(&z, sub_matches).await;
        }
        Some(("subscribe", sub_matches)) => {
            action::do_subscribe(&z, sub_matches).await;
        }
        Some(("query", sub_matches)) => {
            action::do_query(&z, sub_matches).await;
        }
        _ => {}
    }
}
