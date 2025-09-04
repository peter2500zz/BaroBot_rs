use kovi::build_bot;

fn main() {
    build_bot!(
        // kovi_plugin_cmd,

        // 配置文件
        config,

        // 基于Brigadier的插件
        brigadier,
            ping,
            live_reminder,
            self_info,

        auto_shutup,
    ).run();
}

#[test]
fn get_config() {
    let config = std::sync::Arc::clone(&config::CONFIG);

    println!("{:#?}", config);
}

