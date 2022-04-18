use chrono::{self, Local};
use table_maker::create_table;
use clap::Parser;

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    ///Create a new table
    #[clap(short,long)]
    create:bool,
    
    ///Parse created table and send message to fitting number
    #[clap(short,long)]
    parse:bool,

}
fn main() {
    let cli = Cli::parse();
    if cli.create&&cli.parse{
        eprintln!("Invalid arguments");
        std::process::exit(1);
    }
    println!("Hello, world!");
    let table =
        match create_table(Local::now().date().naive_local(), ***REMOVED***, true) {
            Ok(x) => {x},
            Err(e) => {panic!("{:?}",e)},
        };
    println!("{:?}", table);
}
