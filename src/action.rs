use crate::parser::*;
use clap::ArgMatches;
use colored::Colorize;
use std::any::Any;
use std::borrow::Cow;
use std::time::Duration;
use zenoh::bytes::Encoding;
use zenoh::config::WhatAmI;
use zenoh::query::{ConsolidationMode, QueryTarget};
use zenoh::session::ZenohId;

pub async fn do_scout(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let scout_interval = resolve_argument::<u64>(sub_matches, "SCOUT_INTERVAL", false)
        .await
        .expect("Scout interval should be an integer");

    let config = z.config().lock().clone();
    let scouted = zenoh::scout(WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client, config)
        .await
        .expect("Unable to scout");

    let mut known_nodes = std::collections::HashMap::<ZenohId, bool>::new();
    known_nodes.insert(z.zid(), true);
    let mut sn = 0;

    let _ = tokio::time::timeout(Duration::from_secs(scout_interval), async {
        while let Ok(hello) = scouted.recv_async().await {
            if let std::collections::hash_map::Entry::Vacant(e) = known_nodes.entry(hello.zid()) {
                sn += 1;
                e.insert(true);
                println!("{}({}):", "scouted".bold(), sn);
                println!("\t{}: {}", "Zenoh ID".bold(), hello.zid());
                println!("\t{}: {}", "Kind".bold(), hello.whatami());
                println!(
                    "\t{}:{}\n",
                    "Locators".bold(),
                    hello
                        .locators()
                        .iter()
                        .fold("".to_string(), |a, l| { a + &l.to_string() + ",\n\t   " })
                );
            }
        }
    })
    .await;
}
pub async fn do_publish(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let file_based_data = resolve_bool_argument(sub_matches, "file");

    let kexpr: String = resolve_argument(sub_matches, "KEY_EXPR", false)
        .await
        .unwrap();

    let value: String = resolve_argument(sub_matches, "VALUE", file_based_data)
        .await
        .unwrap();

    let some_attach =
        resolve_optional_argument::<String>(sub_matches, "ATTACHMENT", file_based_data)
            .await
            .unwrap();

    let count = *resolve_optional_argument::<u32>(sub_matches, "count", false)
        .await
        .unwrap()
        .get_or_insert(1);

    let period = *resolve_optional_argument::<u64>(sub_matches, "period", false)
        .await
        .unwrap()
        .get_or_insert(0);

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
    let kexpr: String = resolve_argument(sub_matches, "KEY_EXPR", false)
        .await
        .unwrap();

    let s = z.declare_subscriber(kexpr).await.unwrap();
    let mut n: u64 = 0;
    while let Ok(sample) = s.recv_async().await {
        n += 1;
        println!("{}({}):", "Sample".bold(), n);
        if sample.encoding().type_id() == zenoh::bytes::Encoding::ZENOH_STRING.type_id() {
            if let Some(attch) = sample.attachment() {
                let str = attch.try_to_string().unwrap_or(Cow::from("[..]"));

                println!(
                    "\t{}: {}\n\t{}: {}\n\t{}: {}\n",
                    "key".bold(),
                    sample.key_expr(),
                    "value".bold(),
                    sample.payload().try_to_string().unwrap(),
                    "attachment".bold(),
                    str
                );
            } else {
                println!(
                    "\t{}: {}\n\t{}: {}\n",
                    "key".bold(),
                    sample.key_expr(),
                    "value".bold(),
                    sample.payload().try_to_string().unwrap()
                );
            }
        }
    }
}

pub(crate) async fn do_query(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let file_based_data = resolve_bool_argument(sub_matches, "file");

    let qexpr: String = resolve_argument(sub_matches, "QUERY_EXPR", false)
        .await
        .unwrap();

    let body = resolve_optional_argument::<String>(sub_matches, "BODY", file_based_data)
        .await
        .unwrap();

    let target = match resolve_optional_argument::<String>(sub_matches, "target", false)
        .await
        .unwrap()
        .unwrap_or("best".into())
        .as_str()
    {
        "best" => QueryTarget::BestMatching,
        "all" => QueryTarget::All,
        "all-complete" => QueryTarget::AllComplete,
        _ => {
            println!("Ignoring invalid target applying: BestMatching");
            QueryTarget::BestMatching
        }
    };
    println!("Query target: {:?}", target);

    let consolidation =
        match resolve_optional_argument::<String>(sub_matches, "consolidation", false)
            .await
            .unwrap()
            .unwrap_or("none".into())
            .as_str()
        {
            "none" => ConsolidationMode::None,
            "monotonic" => ConsolidationMode::Monotonic,
            "latest" => ConsolidationMode::Latest,
            _ => {
                println!("Ignoring invalid target applying: BestMatching");
                ConsolidationMode::None
            }
        };
    println!("Consolidation mode: {:?}", consolidation);
    let some_attach =
        resolve_optional_argument::<String>(sub_matches, "ATTACHMENT", file_based_data)
            .await
            .unwrap();

    let replies = if body.is_none() {
        if some_attach.is_none() {
            z.get(qexpr)
                .target(target)
                .consolidation(consolidation)
                .await
                .unwrap()
        } else {
            z.get(qexpr)
                .target(target)
                .consolidation(consolidation)
                .attachment(some_attach.unwrap())
                .await
                .unwrap()
        }
    } else if some_attach.is_none() {
        z.get(qexpr)
            .target(target)
            .consolidation(consolidation)
            .payload(body.unwrap())
            .await
            .unwrap()
    } else {
        z.get(qexpr)
            .target(target)
            .consolidation(consolidation)
            .payload(body.unwrap())
            .attachment(some_attach.unwrap())
            .await
            .unwrap()
    };

    let mut count: u64 = 0;
    while let Ok(reply) = replies.recv_async().await {
        count += 1;
        println!("{}({}):", "Reply".bold(), count);
        let rid = if let Some(id) = reply.replier_id() {
            id.to_string()
        } else {
            "Unknown".into()
        };
        println!("\t{}: {}", "Replier Id".bold(), rid);

        match reply.result() {
            Ok(result) => {
                let sid = if let Some(sid) = result.source_info().source_id() {
                    sid.zid().to_string()
                } else {
                    "Unknown".into()
                };
                let ssn = result.source_info().source_sn().unwrap_or_default();

                println!("\t{}: {}", "Source Id".bold(), sid);
                println!("\t{}: {}", "Source SN".bold(), ssn);
                println!("\t{}: {}", "Key".bold(), result.key_expr());
                println!(
                    "\t{}: {}",
                    "Value".bold(),
                    result.payload().try_to_string().unwrap()
                );
            }
            Err(e) => {
                println!("\t{}: {}", "Result".bold(), e);
            }
        }
    }
}
