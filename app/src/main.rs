// use crate::main
use countdown_bot3::countdown_bot::bot;
use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;
#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().expect("Cannot get current working dir!");
    println!("Working dir: {}", &cwd.display());
    let mut bot = bot::CountdownBot::new(&cwd);
    bot.add_plugin_static_register_hook(demo::plugin_register);
    bot.add_plugin_static_register_hook(simple_rand::plugin_register);
    bot.add_plugin_static_register_hook(weather::plugin_register);
    bot.add_plugin_static_register_hook(couplet::plugin_register);
    bot.add_plugin_static_register_hook(broadcast::plugin_register);
    bot.add_plugin_static_register_hook(group_noticer::plugin_register);
    bot.add_plugin_static_register_hook(hitokoto::plugin_register);
    bot.add_plugin_static_register_hook(fun::plugin_register);
    
    bot.init().await.expect("Failed to initialize bot.");
    bot.run().await.unwrap();
}
