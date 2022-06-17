mod heb_cal;
mod list;
pub mod table_construction;

pub use heb_cal::{HebDate, HebDateRaw};
pub use list::Person;
pub use table_construction::construction::{self, NamesTableRaw};

pub fn create_table(exclude_dates: bool) -> Result<String, Box<dyn std::error::Error>> {
    construction::create_table(exclude_dates)
}
