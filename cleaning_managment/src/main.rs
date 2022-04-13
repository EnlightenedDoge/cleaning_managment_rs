use table_maker::generate_heb_json;

fn main() {
    println!("Hello, world!");
    generate_heb_json().expect("Error reading table");
}
