use clap;
use geister_ai;
use geister_core::player::*;
use geister_gpw_proto::run_client;
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
    if let Some(_m) = args.subcommand_matches("random") {
        let mut ai = geister_ai::RandomAi::new(id);
        println!("run-client");
        run_client(&mut ai).unwrap();
    }
}
