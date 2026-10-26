//! Reproducible library work only, without rendering, audio or an emulator.
//! cargo run --release -p slot --no-default-features --features sim --example library_bench -- /path/to/Slot
use std::{hint::black_box, path::PathBuf, time::Instant};
use slot::{app::App, root};
use slot_input::{Action, Btn};

fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("pass a fixture Slot folder"));
    root::ensure(&root);
    let carts = slot_store::scan(&root).expect("scan fixture");
    println!("games={}", carts.len());
    for run in 0..5 {
        let start = Instant::now();
        let mut app = App::new(carts.clone());
        let create = start.elapsed();
        let start = Instant::now();
        app.search_games("");
        let search = start.elapsed();
        let start = Instant::now();
        app.apply(Action::GbaDown(Btn::B));
        app.apply(Action::GbaUp(Btn::B));
        let back = start.elapsed();
        black_box(app);
        println!("run={run} create_ms={:.3} search_all_ms={:.3} back_ms={:.3}", create.as_secs_f64()*1000.0, search.as_secs_f64()*1000.0, back.as_secs_f64()*1000.0);
    }
}
