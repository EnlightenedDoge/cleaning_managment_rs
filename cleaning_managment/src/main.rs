use clap::Parser;
use table_maker::create_table;
use table_reader::sender;

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    ///Create a new table
    #[clap(short, long)]
    create: bool,

    ///Parse created table and send message to fitting number
    #[clap(short, long)]
    parse: bool,
}
fn main() {
    let cli = Cli::parse();
    if cli.create && cli.parse {
        eprintln!("Invalid arguments");
        std::process::exit(1);
    }
    if cli.create {
        let table = match create_table(true) {
            Ok(x) => x,
            Err(e) => {
                panic!("{:?}", e)
            }
        };
        println!("{:?}", table);
    } else {
        println!(
            "{:?}",
            sender::send_to("***REMOVED***", "Test").unwrap() //table::get_soldiers_table("./output/output_table.csv").unwrap()
        );
    }
}
