pub mod command_options;
pub use command_options::CommandOptions as CommandOptions;

pub mod config_json;
pub use config_json::ConfigJson as ConfigJson;

pub fn run(opt: CommandOptions, _cfg: ConfigJson) -> Result<(), String> {
    if opt.usage {
        println!("{}", opt.usage());
        return Ok(());
    }

    Err(String::from("TBD"))
}
