use kovi::build_bot;

fn main() {
    build_bot!(
        // kovi_plugin_cmd,
        // baro_main,
        brigadier,
        ping,
        live_reminder,
    ).run();
}

#[test]
fn get_config() {
    let config = std::sync::Arc::clone(&config::CONFIG);

    println!("{:#?}", config);
}

