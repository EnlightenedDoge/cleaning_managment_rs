use std::{path::Path, process::exit};

use clap::Parser;
use table_configs::paths::{self, get_config_path, get_root_dir_path, init};
use table_maker::create_table;

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    ///Create a new table
    #[clap(short, long)]
    create: bool,

    ///Parse created table and send message to fitting number
    #[clap(short, long)]
    parse: bool,

    ///Clean config files from their folders. Run this when you want to uninstall.
    #[clap(short, long)]
    remove: bool,
}
fn main() {
    match init() {
        Ok(is_init) => {
            if !is_init {
                let path = get_config_path();
                let path = Path::new(&path).parent().unwrap();
                println!("Config files missing. Check \"{}\"", path.to_str().unwrap());
                exit(0)
            }
        }
        Err(_) => {
            panic!(
                "Failed to create/find specified path \"{:?}\".",
                get_root_dir_path()
            );
        }
    }
    let cli = Cli::parse();
    if cli.create && cli.parse {
        eprintln!("Invalid arguments");
        std::process::exit(1);
    }
    if cli.remove && !(cli.create || cli.parse) {
        std::fs::remove_dir_all(&paths::get_root_dir_path()).expect(&format!(
            "Could not remove config files from: {}",
            &paths::get_root_dir_path()
        ));
        println!(
            "Files removed successfully from: {}",
            &paths::get_root_dir_path()
        );
        exit(0);
    }
    if cli.create {
        let table = match create_table(true) {
            Ok(x) => x,
            Err(e) => {
                if e.is::<std::io::Error>() {
                    panic!("Could not read or write to one of the files. Check for missing files.\n{:?}", e)
                }
                panic!("{:?}", e)
            }
        };
        println!("{}", &table);
    } else {
        match table_reader::start_interface() {
            Ok(_) => {}
            Err(e) => {
                if e.is::<std::io::Error>() {
                    panic!("Could not read or write to one of the files. Check for missing files.\n{:?}", e)
                }
                panic!("{:?}", e)
            }
        }
        // println!(
        //     "{:?}",
        //     sender::send_to("***REMOVED***", "Test").unwrap() //table::get_soldiers_table("./output/output_table.csv").unwrap()
        // );
    }
}
