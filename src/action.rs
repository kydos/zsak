use std::any::Any;
use std::borrow::Cow;
use clap::ArgMatches;
use zenoh::bytes::Encoding;
use colored::Colorize;
use tokio::io::AsyncReadExt;

pub async fn do_put(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let file_based_data = if let Some(b) =
        sub_matches.get_one::<bool>("file") { *b} else { false };

    let kexpr = sub_matches.get_one::<String>("KEY_EXPR").unwrap();

    let t_value = if file_based_data {
        let fname = sub_matches.get_one::<String>("VALUE").unwrap();
        let mut f = tokio::fs::File::open(fname).await.expect("Unable to open file");
        let mut content = String::new();
        let _ = f.read_to_string(&mut content).await.expect("Unable to read file");
        content
    } else {
        sub_matches.get_one::<String>("VALUE").unwrap().into()
    };

    let some_attch: Option<String> =
        if file_based_data {
            if let Some(fname) = sub_matches.get_one::<String>("ATTACHMENT") {
                let mut f = tokio::fs::File::open(fname).await.expect("Unable to open file");
                let mut content = String::new();
                let _ = f.read_to_string(&mut content).await.expect("Unable to read file");
                Some(content)
            } else { None }
        } else {
            sub_matches.get_one::<String>("ATTACHMENT").map(|s| s.to_string())
        };


    let count =
        if let Some(s) = sub_matches.get_one::<String>("count") {
            s.parse::<u32>().expect("the COUNT should be an integer")
        } else { 1 };
    let period =
        if let Some(s) = sub_matches.get_one::<String>("period") {
            s.parse::<u64>().expect("the DURATION should be an integer")
        } else { 0 };


    for i in 1..= count {
        let value = t_value.replace("{N}", i.to_string().as_str());
        if some_attch.is_none() {
            z.put(kexpr, value).encoding(Encoding::ZENOH_STRING).await.unwrap();
            println!("[{}]", i);

        } else {
            z.put(kexpr, value).attachment(some_attch.clone().unwrap()).encoding(Encoding::ZENOH_STRING).await.unwrap();
            println!("[{}]", i);
        }
        if period != 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(period)).await;
        }
    }

}

pub async fn do_sub(z: &zenoh::Session, sub_matches: &ArgMatches) {
    let kexpr = sub_matches.get_one::<String>("KEY_EXPR").unwrap();

    let s = z.declare_subscriber(kexpr).await.unwrap();
    while let  Ok(sample) = s.recv_async().await {
        if sample.encoding().type_id() == zenoh::bytes::Encoding::ZENOH_STRING.type_id() {
            if let Some(attch) = sample.attachment() {
                let str = attch.try_to_string().unwrap_or(Cow::from("[..]"));

                println!("{}: {}\n{}: {}\n{}: {}\n",
                         "key".bold(), sample.key_expr(),
                         "value".bold(), sample.payload().try_to_string().unwrap(),
                         "attachment".bold(), str);
            } else {
                println!("{}: {}\n{}: {}\n",
                         "key".bold(), sample.key_expr(),
                         "value".bold(), sample.payload().try_to_string().unwrap());
            }
        }
    }
}
