// use crate::main
use countdown_bot3::countdown_bot::bot;
use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;
#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().expect("Cannot get current working dir!");
    let path = cwd;
    println!("Working dir: {}", path.display());
    let mut bot = bot::CountdownBot::new(&path);
    bot.init().expect("?");
    bot.start().await.unwrap();
    // return Ok(());
}
