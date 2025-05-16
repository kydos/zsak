use clap::{ArgMatches, Command, arg};
use std::str::FromStr;
use tokio::io::AsyncReadExt;

const PUB_AFTER_HELP: &str = r#"
To simply publish a value for a given key you can do as follows:

    zenoh publish greeting hello

You can also publish multiple messages by specifying the --count option and using the {N} macro
if you want to diplay the cardinal number of the message:

    zenoh publish --count 10 "This is the {N}th time I am saying hello!"

You can also publish messages periodically, by specifiyng a duration in milliseconds:

    zenoh publish --count 10 --period 1000 "This is the {N}th time I am saying hello -- every second!"

"#;

const SUB_AFTER_HELP: &str = r#"
Creating a subscriber is extremely easy, as shown below:

    zenoh subscribe zenoh/greeting

You can also use key expressions, as in:

    zenoh subscribe zenoh/*

"#;

const QUERY_AFTER_HELP: &str = r#"
ADD HERE FEW EXAMPLES
"#;

const QUERYABLE_AFTER_HELP: &str = r#"
ADD HERE FEW EXAMPLES
"#;

const STORAGE_AFTER_HELP: &str = r#"
ADD HERE FEW EXAMPLES
"#;

pub(crate) fn arg_parser() -> Command {
    let cmd =
        Command::new("zenoh")
            .about("Command line tool for publishing, subscribing and quering in Zenoh")
            .subcommand_required(true)
            .arg(arg!(-c --config <KEY_EXPR> "The Zenoh configuration").required(false))
            .arg(arg!(-a --admin "Enables the admin space").required(false))
            .arg(arg!(-r --rest <PORT> "Enables the REST plugin").required(false))
            .arg(arg!(-m --mode <MODE> "The application mode <client|peer|router>").required(false))
            .arg(arg!(-e --endpoints <ENDPOINTS> "The array of endpoints to connect to, e.g. [\"tcp/10.0.0.1:7447\", \"udp/10.0.0.1:7444\"]").required(false))
            .arg(arg!(--no-multicast-scouting "Disable multicast Scouting").required(false))
            .arg(arg!(-n --name <NAME> "Name for the application").required(false))

            .subcommand(
                Command::new("doctor")
                    .about("Checks the environment to see if zenohd and the required plugins are available")
            )
            .subcommand(
                Command::new("scout")
                    .about("Scouts Zenoh runtimes on the network")
                    .arg(arg!(<SCOUT_INTERVAL> "The time in seconds during which Zenoh will scout.").required(true))
            )
            .subcommand(
                Command::new("publish")
                    .about("Publishes data on a given key expression")
                    .arg(arg!(-c --count <NUMBER> "The number of publications").required(false))
                    .arg(arg!(-p --period <DURATION> "The period of publications").required(false))
                    .arg(arg!(-u --unreliable  "Asks to send this data with the best-effot QoS").required(false))
                    .arg(arg!(--priority <PRIO>  "Set the specific priority").required(false))
                    .arg(arg!(-f --file "If enabled expects that value/attachment are file names").required(false))
                    .arg(arg!(<KEY_EXPR> "The key expression used for the publication").required(true))
                    .arg(arg!(<VALUE> "The value used for this publication").required(true))
                    .arg(arg!(<ATTACHMENT> "The publication attachment, if any").required(false))
                    .after_help(PUB_AFTER_HELP),
            )
            .subcommand(
                Command::new("subscribe")
                    .about("Subscribe to the given key expression")
                    .arg(arg!(<KEY_EXPR> "The key expression used for the publication").required(true))
                    .after_help(SUB_AFTER_HELP),
            )
            .subcommand(
                Command::new("query")
                    .about("Issues a query")
                    .arg(arg!(-f --file "If enabled expects that body/attachment are file names").required(false))
                    .arg(arg!(-t --target <QUERY_TARGET> "Should be one of <best|all|all-complete>, \"best\" used by as the default.").required(false))
                    .arg(arg!(-c --consolidation <CONSOLIDATION> "Should be one of <none|monotonic|latest>,  \"none\" used as the default.").required(false))
                    .arg(arg!(<QUERY_EXPR> "The key expression used for the publication").required(true))
                    .arg(arg!(<BODY> "The value used for this publication").required(false))
                    .arg(arg!(<ATTACHMENT> "The publication attachment, if any").required(false))
                    .after_help(QUERY_AFTER_HELP),
            )
            .subcommand(
                Command::new("queryable")
                    .about("Declares a queryable answering queries for a given key expression")
                    .arg(arg!(-c --complete "Makes the queriable complete").required(false))
                    .arg(arg!(-f --file "If enabled expects that the replt is a file name").required(false))
                    .arg(arg!(-s --script "the <REPLY> is a python script").required(false))
                    .arg(arg!(-p --packages <PATH> "The path to the python site-packages including required dependencies").required(false))
                    .arg(arg!(<KEY_EXPR> "The expression associated with the queryable"))
                    .arg(arg!(<REPLY> "The reply used by the queryable"))
                    .after_help(QUERYABLE_AFTER_HELP)
            )
            .subcommand(
                Command::new("storage")
                    .about("Declares an in-memory storage")
                    .arg(arg!(-c --complete "Makes the queriable complete").required(false))
                    .arg(arg!(-a --align "Automatically align storages").required(false))
                    .arg(arg!(<KEY_EXPR> "The expression associated with the queryable"))
                    .after_help(STORAGE_AFTER_HELP)
            );

    if cfg!(feature = "video") {
        let video_cmd = Command::new("stream")
            .about("Starts streaming video from the selected camera with the provided options")
            .arg(arg!(-r --resolution <RESOLUTION> "The video resolution <widthxheight>, the default is 1280x720 (HD)"));
        cmd.subcommand(video_cmd)
    } else {
        cmd
    }
}

pub(crate) async fn resolve_argument<T: FromStr>(
    sub_matches: &ArgMatches,
    arg: &str,
    file_based: bool,
) -> Result<T, T::Err> {
    let v = sub_matches.get_one::<String>(arg).unwrap();
    if file_based {
        let mut f = tokio::fs::File::open(v).await.expect("Unable to open file");
        let mut content = String::new();
        let _ = f
            .read_to_string(&mut content)
            .await
            .expect("Unable to read file");
        content.parse::<T>()
    } else {
        v.to_string().parse::<T>()
    }
}
pub(crate) async fn resolve_optional_argument<T: FromStr>(
    sub_matches: &ArgMatches,
    arg: &str,
    file_based: bool,
) -> Result<Option<T>, T::Err> {
    if let Some(v) = sub_matches.get_one::<String>(arg) {
        if file_based {
            let mut f = tokio::fs::File::open(v).await.expect("Unable to open file");
            let mut content = String::new();
            let _ = f
                .read_to_string(&mut content)
                .await
                .expect("Unable to read file");
            content.parse::<T>().map(Some)
        } else {
            v.to_string().parse::<T>().map(Some)
        }
    } else {
        Ok(None)
    }
}
pub(crate) fn resolve_bool_argument(sub_matches: &ArgMatches, arg: &str) -> bool {
    if let Some(v) = sub_matches.get_one::<bool>(arg) {
        *v
    } else {
        false
    }
}
