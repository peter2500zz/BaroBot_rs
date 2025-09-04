mod config;

// use kovi::PluginBuilder as plugin;
use std::sync::{Arc, LazyLock};

pub use config::*;

pub const CONFIG: LazyLock<Arc<Config>> = LazyLock::new(|| {
    Arc::new(Config::from_yaml("config.yaml").unwrap_or_default())
});

#[kovi::plugin]
async fn main() {
    // plugin::on_msg(|event| async move {
    //     if event.borrow_text() == Some("hi") {
    //         event.reply("hi")
    //     }
    // });
}
