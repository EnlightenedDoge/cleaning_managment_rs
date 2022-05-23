mod heb_cal;
mod list;
pub mod table_construction;

pub use heb_cal::{HebDate, HebDateRaw};
pub use list::Soldier;
pub use table_construction::construction::{self, NamesTableRaw};

// const EXCLUDED_DATES_PATH: &str = "./config/excluded_hebcal.csv";
// const SOLDIERS_PATH: &str = "./config/names.csv";
// const HEBDATE_PATH: &str = "./config/heb_date.json";
// pub const CONFIG_PATH: &str = "./config/config.json";

pub fn create_table(exclude_dates: bool) -> Result<String, Box<dyn std::error::Error>> {
    construction::create_table(exclude_dates)
}
