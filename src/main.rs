mod srt_parser;
mod translator;
mod exec;
use tokio;
use clap::{Parser, Subcommand};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();

    if cli.verbose {
        println!("Running in verbose mode");
    }

    match cli.command {
        Some(Commands::Translate { path }) => {
            let res = exec::translate(path).await;
            match res {
                Ok(()) => {println!("Translation sucessfull !");}
                Err(_e) => {println!("An error has occured during translation");}
            }
        }
        None => {
            println!("No command provided. Use --help to see available options.");
        }
    }


}



//clap 



#[derive(Parser)]
#[command(name = "SRT translator")]
#[command(about = "A simple CLI to translate sr tfiles using LLM", long_about = None)]
struct Cli {
    /// The verbose flag
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Greet a person by name
    Translate { path: String },
}