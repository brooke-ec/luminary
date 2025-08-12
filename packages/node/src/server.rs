use eyre::Result;
use luminary_macros::command;
use serde_json::json;

struct CommandContext {}

fn main() -> Result<()> {
    PingCommand { time: 42 };
    let _ = ping(CommandContext {}, json!({"time": 42}));
    return Ok(());
}

#[command]
fn ping(_ctx: CommandContext, time: u64) -> Result<()> {
    println!("Ping received! {:}", time);
    return Ok(());
}
