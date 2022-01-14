use countdown_bot3::countdown_bot::bot;
use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;
#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().expect("Cannot get current working dir!");
    println!("Working dir: {}", &cwd.display());
    let mut bot = bot::CountdownBot::new(&cwd);
    bot.add_plugin_static_register_hook(simple_rand::plugin_register);
    bot.add_plugin_static_register_hook(weather::plugin_register);
    bot.add_plugin_static_register_hook(couplet::plugin_register);
    bot.add_plugin_static_register_hook(broadcast::plugin_register);
    bot.add_plugin_static_register_hook(group_noticer::plugin_register);
    bot.add_plugin_static_register_hook(hitokoto::plugin_register);
    bot.add_plugin_static_register_hook(fun::plugin_register);
    bot.add_plugin_static_register_hook(oierdb_query::plugin_register);
    bot.add_plugin_static_register_hook(oiwiki_query::plugin_register);
    bot.add_plugin_static_register_hook(qrcode_make::plugin_register);
    bot.add_plugin_static_register_hook(dns_query::plugin_register);
    bot.add_plugin_static_register_hook(music_163::plugin_register);
    bot.add_plugin_static_register_hook(jiuge::plugin_register);
    bot.add_plugin_static_register_hook(covid19::plugin_register);
    bot.add_plugin_static_register_hook(bullshit::plugin_register);
    bot.add_plugin_static_register_hook(ds_drawer_plugin::plugin_register);

    bot.init().await.expect("Failed to initialize bot.");
    bot.run().await.unwrap();
}
