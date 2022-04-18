mod heb_cal;
mod list;
use chrono::NaiveDate;
use heb_cal::exclued_holidays_from_file;
use heb_cal::generate_heb;
use list::*;
use csv::Writer;
use serde::Serialize;
const HEBDATE_PATH: &str = "./config/heb_date.json";
const EXCLUDED_DATES_PATH: &str = "./config/excluded_hebcal.json";
const SOLDIERS_PATH: &str = "./config/NameSheet.json";
const TABLE_OUTPUT: &str = "./output/output_table.csv";


pub fn create_table(
    start_date: NaiveDate,
    time_period: usize,
    exclude_dates: bool,
) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let mut heb_cal = generate_heb()?;
    if exclude_dates {
        heb_cal = exclued_holidays_from_file(heb_cal, EXCLUDED_DATES_PATH)?;
    }
    let soldiers = list::parse_soldiers_from_file(SOLDIERS_PATH)?;
    let dates = get_dates(soldiers, heb_cal, start_date, time_period);
    let mut wtr = Writer::from_writer(vec![]);
    for date in dates.iter(){
        wtr.serialize(Raw{
            name:&date.soldier.name,
            date:&date.date.to_string(),
            number:&date.soldier.phone,
        })?;
    }
    std::fs::write(TABLE_OUTPUT, String::from_utf8(wtr.into_inner()?)?)?;
    Ok(dates)
}

#[derive(Serialize)]
struct Raw<'a>{
    name:&'a str,
    number:&'a str,
    date:&'a str,
}