use crate::parser::*;

use clap::ArgMatches;
use colored::Colorize;
use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;
use zenoh::bytes::Encoding;
use zenoh::config::WhatAmI;
use zenoh::liveliness::LivelinessToken;
use zenoh::qos::Reliability;
use zenoh::query::{ConsolidationMode, QueryTarget};
use zenoh::sample::{SampleKind, SourceInfo};
use zenoh::session::ZenohId;

const LIST_SCOUTING_INTERVAL: u64 = 2;

pub async fn do_doctor() {
    match std::env::var("ZSAK_HOME") {
        Ok(_) => {
            if tokio::process::Command::new("zenohd")
                .arg("-h")
                .output()
                .await
                .is_err()
            {
                println!(
                    "There is no zenohd defined in your PATH. Please double check your system configuration."
                );
            }
        }
        Err(_) => {
            println!(
                "The ZSAK_HOME environment variable is not set, please run the setup.sh (see README.md for more information)"
            );
        }
    };
}

pub async fn do_scout(
    z: &zenoh::Session,
    scout_interval: u64,
) -> HashMap<ZenohId, zenoh::scouting::Hello> {
    let config = z.config().lock().clone();
    let scouted = zenoh::scout(WhatAmI::Router | WhatAmI::Peer | WhatAmI::Client, config)
        .await
        .expect("Unable to scout");

    let mut known_nodes = HashMap::<ZenohId, zenoh::scouting::Hello>::new();

    let _ = tokio::time::timeout(Duration::from_secs(scout_interval), async {
        while let Ok(hello) = scouted.recv_async().await {
            known_nodes.insert(hello.zid(), hello);
        }
    })
    .await;

    known_nodes.remove(&z.zid());
    known_nodes
}
pub async fn do_list(z: &zenoh::Session, kind: usize) -> Vec<(String, WhatAmI)> {
    do_scout(z, LIST_SCOUTING_INTERVAL)
        .await
        .iter()
        .filter(|(_, h)| ((h.whatami() as usize) & kind != 0))
        .map(|(zid, hello)| (zid.to_string(), hello.whatami()))
        .collect::<Vec<(String, WhatAmI)>>()
}
pub async fn do_publish(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let reliability = if resolve_bool_argument(sub_matches, "unreliable") {
        Reliability::BestEffort
    } else {
        Reliability::Reliable
    };

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
                .reliability(reliability)
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
            tokio::time::sleep(Duration::from_millis(period)).await;
        }
    }
}

pub async fn do_delete(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let kexpr: String = resolve_argument(sub_matches, "KEY_EXPR", false)
        .await
        .unwrap();

    z.delete(kexpr).await.unwrap();
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
        if sample.encoding().type_id() == Encoding::ZENOH_STRING.type_id() {
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
                    "Timestamp".bold(),
                    result
                        .timestamp()
                        .map(|ts| { ts.to_string() })
                        .unwrap_or_else(|| "None".into())
                );
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

#[cfg(feature = "video")]
use zenoh::shm::{
    POSIX_PROTOCOL_ID, PosixShmProviderBackend, PosixShmProviderBackendBuilder, ShmProvider,
    ShmProviderBuilder, StaticProtocolID,
};

#[cfg(feature = "video")]
const SHM_BUF_SIZE: usize = 64 * 1024 * 1024;

#[cfg(feature = "video")]
pub(crate) async fn do_stream(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let res = resolve_argument::<String>(sub_matches, "RESOLUTION", false)
        .await
        .unwrap();
    let with_height: Vec<usize> = res
        .split('x')
        .map(|p| p.parse::<usize>().unwrap())
        .collect();

    let shm_back_end = zenoh::shm::PosixShmProviderBackend::builder()
        .with_size(SHM_BUF_SIZE)
        .unwrap()
        .wait()
        .unwrap();
    let shm_provider: ShmProvider<StaticProtocolID<0>, PosixShmProviderBackend> =
        ShmProviderBuilder::builder()
            .protocol_id::<POSIX_PROTOCOL_ID>()
            .backend(shm_back_end)
            .wait();

    // let (config, key_expr, resolution, delay, reliability, congestion_ctrl, image_quality) =
    //     parse_args();

    println!("Opening session...");
    let z = zenoh::open(config).await.unwrap();

    let publ = z
        .declare_publisher(&key_expr)
        .reliability(reliability)
        .congestion_control(congestion_ctrl)
        .await
        .unwrap();

    let conf_sub = z
        .declare_subscriber(format!("{}/zcapture/conf/**", key_expr))
        .await
        .unwrap();

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();

    let opened = videoio::VideoCapture::is_opened(&cam).unwrap();

    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut encode_options = opencv::core::Vector::<i32>::new();
    encode_options.push(opencv::imgcodecs::IMWRITE_JPEG_QUALITY);
    encode_options.push(image_quality);

    loop {
        select!(
            _ = tokio::time::sleep(std::time::Duration::from_millis(delay)).fuse() => {
                let mut frame = Mat::default();
                cam.read(&mut frame).unwrap();

                if !frame.empty() {
                    let mut reduced = Mat::default();
                    opencv::imgproc::resize(&frame, &mut reduced, opencv::core::Size::new(resolution[0], resolution[1]), 0.0, 0.0 , opencv::imgproc::INTER_LINEAR).unwrap();

                    let mut buf = opencv::core::Vector::<u8>::new();
                    opencv::imgcodecs::imencode(".jpeg", &reduced, &mut buf, &encode_options).unwrap();
                    let shm_len = ((buf.len() >> 8) + 1) << 8;
                    let mut shm_buf = shm_provider
                        .alloc(shm_len)
                        .with_policy::<BlockOn<Defragment<GarbageCollect>>>()
                        .wait().expect("Failed to allocate SHM buffer");
                    let bs = buf.to_vec();
                    for i in 0.. bs.len() {
                        shm_buf[i] = bs[i];
                    }
                    publ.put(shm_buf).wait().unwrap();
                } else {
                    println!("Reading empty buffer from camera... Waiting some more....");
                }
            },
            sample = conf_sub.recv_async().fuse() => {
                let sample = sample.unwrap();
                let conf_key = sample.key_expr().as_str().split("/conf/").last().unwrap();
                let conf_val = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
                let _ = z.config().insert_json5(conf_key, &conf_val);
            },
        );
    }
}

pub(crate) async fn do_queryable(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let complete = resolve_bool_argument(sub_matches, "complete");
    let exec_script = resolve_bool_argument(sub_matches, "script");
    let site_packages = if let Some(path) =
        resolve_optional_argument::<String>(sub_matches, "packages", false)
            .await
            .unwrap()
    {
        format!("import sys\nsys.path.append('{}')\n", path)
    } else {
        String::default()
    };

    let file_based = resolve_bool_argument(sub_matches, "file");
    let kexpr: String = resolve_argument(sub_matches, "KEY_EXPR", false)
        .await
        .unwrap();
    let reply: String = resolve_argument(sub_matches, "REPLY", file_based)
        .await
        .unwrap();

    let queryable = z
        .declare_queryable(kexpr.clone())
        .complete(complete)
        .await
        .expect("Unable to declare queryable");

    let mut n = 0;
    let si = SourceInfo::new(Some(queryable.id()), Some(0));
    println!("\tQueryable Running!");
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    use std::ffi::CString;
    pyo3::prepare_freethreaded_python();
    while let Ok(query) = queryable.recv_async().await {
        n += 1;
        println!("{}({}):", "Query".bold(), n);
        println!("\t{}: {}", "Key Expr".bold(), query.key_expr());
        if exec_script {
            let result = pyo3::prelude::Python::with_gil(|py| {
                let locals = PyDict::new(py);
                let key_expr = query.key_expr().to_string();
                let payload = query
                    .payload()
                    .map(|p| p.to_bytes().to_vec())
                    .unwrap_or_else(Vec::new);
                locals.set_item("key_expr", key_expr).unwrap();
                locals.set_item("payload", payload).unwrap();

                let script = CString::new(site_packages.clone() + reply.as_str()).unwrap();
                py.run(script.as_c_str(), None, Some(&locals)).unwrap();
                locals
                    .get_item("result")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap()
            });

            query
                .reply(query.key_expr(), &result)
                .source_info(si.clone())
                .timestamp(z.new_timestamp())
                .await
                .unwrap();
        } else {
            query
                .reply(query.key_expr(), &reply)
                .source_info(si.clone())
                .timestamp(z.new_timestamp())
                .await
                .unwrap();
        }
    }
}
pub(crate) async fn do_declare_liveliness_token(
    z: &zenoh::Session,
    key_expr: &str,
) -> LivelinessToken {
    z.liveliness().declare_token(key_expr).await.unwrap()
}

pub(crate) async fn do_subscribe_liveliness_token(z: &zenoh::Session, key_expr: &str) {
    let sub = z.liveliness().declare_subscriber(key_expr).await.unwrap();
    println!("Join/Leave Events:");
    while let Ok(sample) = sub.recv_async().await {
        match sample.kind() {
            SampleKind::Put => {
                println!(
                    "\t{}: {}",
                    sample.key_expr().as_str().bold(),
                    "Joined".bold().green()
                );
            }
            SampleKind::Delete => {
                println!(
                    "\t{}: {}",
                    sample.key_expr().as_str().bold(),
                    "Left".bold().red()
                );
            }
        }
    }
}

pub(crate) async fn do_query_liveliness(z: &zenoh::Session, key_expr: &str) {
    let replies = z.liveliness().get(key_expr).await.unwrap();
    println!("{}", "Livelines Tokens:".bold());
    while let Ok(reply) = replies.recv_async().await {
        println!(
            "\t- {}",
            reply.result().unwrap().key_expr().as_str().green()
        );
    }
}
