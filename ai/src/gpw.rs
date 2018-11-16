use clap;
use geister_ai;
use geister_core::player::*;
use geister_gpw_proto::run_client;
use std::net::IpAddr;

fn app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("geister-gpwai")
        .version("0.1")
        .author("Yuji Kanagawa <yuji.kngw.80s.revive@gmail.com>")
}

fn main() {
    let args = geister_ai::args(app());
    let id = args.value_of("id").unwrap();
    let id = if id == "1" {
        PlayerID::P1
    } else {
        PlayerID::P2
    };
    let addr: IpAddr = args.value_of("addr").unwrap().parse().expect("Failed to parse Ip address");
    if let Some(_m) = args.subcommand_matches("random") {
        let mut ai = geister_ai::RandomAi::new(id);
        println!("Random AI");
        run_client(&mut ai, addr).unwrap();
    } else if let Some(_m) = args.subcommand_matches("yowagoshi") {
        let mut ai = geister_ai::YowagoshiAi::new(id);
        println!("Yowagoshi AI");
        run_client(&mut ai, addr).unwrap();
    }
}
