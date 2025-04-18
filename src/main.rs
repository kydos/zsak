mod parser;
mod action;


#[tokio::main]
async fn main() {
    let matches = parser::arg_parser().get_matches();

    let config = match matches.get_one::<String>("config") {
        Some(fname) => {
            zenoh::Config::from_file(fname).expect("Unable to open the Zenoh Config")
        },
        None => zenoh::Config::default()
    };
    let z = zenoh::open(config).await.expect("Unable to open the Zenoh Session");

    match matches.subcommand() {
        Some(("pub", sub_matches)) => {
            action::do_put(&z, sub_matches).await;
        },
        Some(("sub", sub_matches)) => {
            action::do_sub(&z, sub_matches).await;
        },
        _ => {

        }
    }
}
