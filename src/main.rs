use kovi::build_bot;

fn main() {
    build_bot!(
        // kovi_plugin_cmd,

        // 配置文件
        config,  // 配置文件前置

        // 基于Brigadier的插件
        brigadier,  // Brigadier命令树解释器
            ping,  // ping -> pong!
            live_reminder,  // Bilibili直播通知器
            self_cmds,  // 基础信息查询
            repeat,  // 复读机

        auto_shutup,  // 定时禁言
        limit_shutup,  // 字数禁言
    ).run();
}
