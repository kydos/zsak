use clap::{Command, arg};

const PUB_AFTER_HELP: &str =
    r#"To simply publish a value for a given key you can do as follows:

     zenoh pub greeting hello

You can also publish multiple messages by specifying the --count option and using the {N} macro
if you want to diplay the cardinal number of the message:

    zenoh pub --count 10 "This is the {N}th time I am saying hello!"

You can also publish messages periodically, by specifiyng a duration in milliseconds:

    zenoh pub --count 10 --period 1000 "This is the {N}th time I am saying hello -- every second!"

"#;

const SUB_AFTER_HELP: &str = r#"
ADD HERE FEW EXAMPLES
"#;


const QUERY_AFTER_HELP: &str = r#"
ADD HERE FEW EXAMPLES
"#;

pub fn arg_parser() -> Command{
    Command::new("zenoh")
        .about("Command line tool for publishing, subscribing and quering in Zenoh")
        .subcommand_required(true)
        .arg(arg!(-c --config <KEY_EXPR> "The Zenoh configuration").required(false))
        .subcommand(
            Command::new("pub")
                .about("Publishes data on a given key expression")
                .arg(arg!(-c --count <NUMBER> "The number of publications").required(false))
                .arg(arg!(-p --period <DURATION> "The number of publications").required(false))
                .arg(arg!(-f --file "If enabled expects that value/attachment are file names"))
                .arg(arg!(<KEY_EXPR> "The key expression used for the publication").required(true))
                .arg(arg!(<VALUE> "The value used for this publication").required(true))
                .arg(arg!(<ATTACHMENT> "The publication attachment, if any").required(false))
                .after_help(PUB_AFTER_HELP)
        )
        .subcommand(
            Command::new("sub")
                .about("Subscribe to the given key expression")
                .arg(arg!(<KEY_EXPR> "The key expression used for the publication").required(true))
                .after_help(SUB_AFTER_HELP)
        )
        .subcommand(
            Command::new("get")
                .about("Issues a query")
                .arg(arg!(<KEY_EXPR> "The key expression used for the publication").required(true))
                .arg(arg!(<BODY> "The value used for this publication").required(false))
                .arg(arg!(<ATTACHMENT> "The publication attachment, if any").required(false))
                .after_help(QUERY_AFTER_HELP)
        )
}
