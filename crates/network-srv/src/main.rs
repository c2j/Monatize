use network_srv::Network;
use std::env;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        println!("network-srv --mock | --url <URL>");
        return;
    }

    if args.iter().any(|a| a == "--mock") {
        println!("network-srv mock OK");
        return;
    }

    let url = args.iter().position(|a| a == "--url").and_then(|i| args.get(i + 1)).cloned();
    if let Some(url) = url {
        let net = Network::new();
        let resp = net.fetch(message_defs::HttpRequest { url }).await.expect("fetch");
        println!("status={} bytes={}", resp.status, resp.body.len());
    } else {
        eprintln!("usage: network-srv --mock | --url <URL>");
        std::process::exit(2);
    }
}

