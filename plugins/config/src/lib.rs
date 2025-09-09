mod config;

// use kovi::PluginBuilder as plugin;
pub use config::*;

pub const CONFIG_PATH: &str = "config.yaml";


#[kovi::plugin]
async fn main() {
    // plugin::on_msg(|event| async move {
    //     if event.borrow_text() == Some("hi") {
    //         event.reply("hi")
    //     }
    // });
}
