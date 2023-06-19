pub mod command_options;

pub use command_options::CommandOptions as CommandOptions;

pub fn run(opt: CommandOptions) -> Result<(), String> {
    if opt.usage {
        println!("{}", opt.usage());
        return Ok(());
    }

    Err(String::from("TDB"))
}
