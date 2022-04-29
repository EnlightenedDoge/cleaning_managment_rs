mod heb_cal;
mod list;
use chrono::Datelike;
use chrono::NaiveDate;
use csv::Writer;
use heb_cal::exclued_holidays_from_file;
use heb_cal::generate_heb;
pub use list::Soldier;
use list::*;
use serde::Deserialize;
use serde::Serialize;
const HEBDATE_PATH: &str = "./config/heb_date.json";
const EXCLUDED_DATES_PATH: &str = "./config/excluded_hebcal.json";
const SOLDIERS_PATH: &str = "./config/NameSheet.json";
pub const CONFIG_PATH: &str = "./config/config.json";

pub fn create_table(exclude_dates: bool) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let config = load_config()?;
    let mut heb_cal = generate_heb()?;
    if exclude_dates {
        heb_cal = exclued_holidays_from_file(heb_cal, EXCLUDED_DATES_PATH)?;
    }
    let mut soldiers = list::parse_soldiers_from_file(SOLDIERS_PATH)?;
    soldiers.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
    let dates = get_dates(soldiers, heb_cal, config.start_date, config.range);
    let mut wtr_raw = Writer::from_writer(vec![]);
    let mut wtr_beaut = Writer::from_writer(vec![]);
    for date in dates.iter() {
        wtr_raw.serialize(Raw {
            name: date.soldier.name.clone(),
            date: date.date.to_string(),
            number: date.soldier.phone.clone(),
        })?;
    }
    for date in dates.iter() {
        wtr_beaut.serialize(BeautyRaw {
            name: date.soldier.name.clone(),
            date: date.date.to_string(),
            day: date.date.weekday().to_string(),
        })?;
    }
    std::fs::write(format!("{}output_table.csv",config.output_path), String::from_utf8(wtr_raw.into_inner()?)?)?;
    std::fs::write(format!("{}beautified_table.csv",config.output_path), String::from_utf8(wtr_beaut.into_inner()?)?)?;
    Ok(dates)
}

fn load_config() -> Result<ConfigMaker, Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(CONFIG_PATH)?;
    let config: ConfigRaw = serde_json::from_str(&config)?;
    let config = ConfigMaker::from(config);
    Ok(config)
}

#[derive(Deserialize)]
pub struct ConfigRaw {
    pub start_date: String,
    pub range: usize,
    pub output_path: String,
    pub send_time: String,
    pub reset_time: String,
}

struct ConfigMaker {
    start_date: NaiveDate,
    range: usize,
    output_path: String,
}
impl ConfigMaker {
    fn from(config: ConfigRaw) -> Self {
        Self {
            output_path: config.output_path,
            range: config.range,
            start_date: NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d").unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Raw {
    pub name: String,
    pub number: String,
    pub date: String,
}

#[derive(Serialize, Deserialize)]
pub struct BeautyRaw{
    pub day:String,
    pub date:String,
    pub name:String,
}