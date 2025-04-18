use clap::ArgMatches;
use colored::Colorize;
use std::any::Any;
use std::borrow::Cow;
use std::time::Duration;
use zenoh::bytes::Encoding;
use zenoh::config::{WhatAmI};
use zenoh::session::ZenohId;

use crate::parser::*;

pub async fn do_scout(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let scout_interval =
        resolve_argument::<u64>(sub_matches, "SCOUT_INTERVAL", false)
            .await
            .expect("Scout interval should be an integer");

    let config = z.config().lock().clone();
    let scouted =
        zenoh::scout(WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client, config).await.expect("Unable to scout");

    let mut known_nodes = std::collections::HashMap::<ZenohId, bool>::new();
    known_nodes.insert(z.zid(), true);
    let mut sn = 0;

    let _ = tokio::time::timeout(Duration::from_secs(scout_interval), async {
        while let Ok(hello) = scouted.recv_async().await {
            if !known_nodes.contains_key(&hello.zid()) {
                sn += 1;
                known_nodes.insert(hello.zid(), true);
                println!("{}({}):", "scouted".bold(), sn);
                println!("\t{}: {}", "Zenoh ID".bold(), hello.zid());
                println!("\t{}: {}", "Kind".bold(), hello.whatami());
                println!("\t{}:{}\n", "Locators".bold(), hello.locators().iter().fold("".to_string(), |a, l| { a + &l.to_string() + ",\n\t   " }));
            }
        }
    }).await;
}
pub async fn do_publish(z: &zenoh::Session, sub_matches: &ArgMatches) {

    let file_based_data =
        if let Some(b) = sub_matches.get_one::<bool>("file") { *b }
        else { false };

    let kexpr: String =
        resolve_argument(sub_matches, "KEY_EXPR", false).await.unwrap();

    let value: String =
        resolve_argument(sub_matches, "VALUE", file_based_data).await.unwrap();

    let some_attach =
        resolve_optional_argument::<String>(sub_matches, "ATTACHMENT", file_based_data).await.unwrap();

    let count =
        *resolve_optional_argument::<u32>(sub_matches, "count", false).await.unwrap().get_or_insert(1);

    let period =
        *resolve_optional_argument::<u64>(sub_matches, "period", false).await.unwrap().get_or_insert(0);


    for i in 1..=count {
        let value = value.replace("{N}", i.to_string().as_str());
        if some_attach.is_none() {
            z.put(&kexpr, value)
                .encoding(Encoding::ZENOH_STRING)
                .await
                .unwrap();
            println!("[{}]", i);
        } else {
            z.put(&kexpr, value)
                .attachment(some_attach.clone().unwrap())
                .encoding(Encoding::ZENOH_STRING)
                .await
                .unwrap();
            println!("[{}]", i);
        }
        if period != 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(period)).await;
        }
    }
}

pub async fn do_subscribe(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let kexpr = sub_matches.get_one::<String>("KEY_EXPR").unwrap();

    let s = z.declare_subscriber(kexpr).await.unwrap();
    while let Ok(sample) = s.recv_async().await {
        if sample.encoding().type_id() == zenoh::bytes::Encoding::ZENOH_STRING.type_id() {
            if let Some(attch) = sample.attachment() {
                let str = attch.try_to_string().unwrap_or(Cow::from("[..]"));

                println!(
                    "{}: {}\n{}: {}\n{}: {}\n",
                    "key".bold(),
                    sample.key_expr(),
                    "value".bold(),
                    sample.payload().try_to_string().unwrap(),
                    "attachment".bold(),
                    str
                );
            } else {
                println!(
                    "{}: {}\n{}: {}\n",
                    "key".bold(),
                    sample.key_expr(),
                    "value".bold(),
                    sample.payload().try_to_string().unwrap()
                );
            }
        }
    }
}



// async fn do_query(z: &zenoh::Session, sub_matches: &ArgMatches) {
//     let file_based_data =
//         if let Some(b) = sub_matches.get_one::<bool>("file") { *b }
//         else { false };
//
//     let qexpr = sub_matches.get_one::<String>("QUERY_EXPR").unwrap();
//
//     let t_body = if file_based_data {
//         let fname = sub_matches.get_one::<String>("BODY").unwrap();
//         let mut f = tokio::fs::File::open(fname)
//             .await
//             .expect("Unable to open file");
//         let mut content = String::new();
//         let _ = f
//             .read_to_string(&mut content)
//             .await
//             .expect("Unable to read file");
//         content
//     } else {
//         sub_matches.get_one::<String>("VALUE").unwrap().into()
//     };
//
//     let some_attch: Option<String> = if file_based_data {
//         if let Some(fname) = sub_matches.get_one::<String>("ATTACHMENT") {
//             let mut f = tokio::fs::File::open(fname)
//                 .await
//                 .expect("Unable to open file");
//             let mut content = String::new();
//             let _ = f
//                 .read_to_string(&mut content)
//                 .await
//                 .expect("Unable to read file");
//             Some(content)
//         } else {
//             None
//         }
//     } else {
//         sub_matches
//             .get_one::<String>("ATTACHMENT")
//             .map(|s| s.to_string())
//     };
//
//     let count = if let Some(s) = sub_matches.get_one::<String>("count") {
//         s.parse::<u32>().expect("the COUNT should be an integer")
//     } else {
//         1
//     };
//     let period = if let Some(s) = sub_matches.get_one::<String>("period") {
//         s.parse::<u64>().expect("the DURATION should be an integer")
//     } else {
//         0
//     };
//
//
//     if some_attch.is_none() {
//         z.put(qexpr, value)
//                 .encoding(Encoding::ZENOH_STRING)
//                 .await
//                 .unwrap();
//             println!("[{}]", i);
//         } else {
//             z.put(qexpr, value)
//                 .attachment(some_attch.clone().unwrap())
//                 .encoding(Encoding::ZENOH_STRING)
//                 .await
//                 .unwrap();
//             println!("[{}]", i);
//         }
//         if period != 0 {
//             tokio::time::sleep(tokio::time::Duration::from_millis(period)).await;
//         }
// }