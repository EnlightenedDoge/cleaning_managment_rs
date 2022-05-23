use std::ops::Deref;

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use serde_json::{self, from_str, Value};
use csv::{self, Reader};

use crate::HEBDATE_PATH;

async fn get_heb_cal() -> Result<String, reqwest::Error> {
    let url = "https://www.hebcal.com/hebcal?v=1&cfg=json&year=now&month=x&maj=on&min=on&mod=on&i=on&geo=none&c=off";
    let str = reqwest::get(url)?.text()?;
    Ok(str)
}

pub fn generate_heb() -> Result<Vec<HebDate>, Box<dyn std::error::Error>> {
    let heb_cal = futures::executor::block_on(get_heb_cal())?;
    std::fs::write(HEBDATE_PATH, &heb_cal)?;

    Ok(get_struct(&heb_cal)?)
}

fn get_struct(json: &str) -> Result<Vec<HebDate>, Box<dyn std::error::Error>> {
    let data = r#json;
    let wrapper: Value = serde_json::from_str(&data)?;
    let wrapper = wrapper["items"].clone();
    let mut items = Vec::<HebDate>::new();
    let array = wrapper.as_array().unwrap();
    let val_to_str = |x: &Value| x.as_str().unwrap().to_string();
    for item in array {
        let a = item.as_object().unwrap();
        items.push(HebDate {
            date: NaiveDate::parse_from_str(a["date"].as_str().unwrap(), "%Y-%m-%d")?,
            title: val_to_str(&a["title"]),
        });
    }
    Ok(items)
}

pub fn exclude_holidays_from_file(
    dates: Vec<HebDate>,
    file: &str,
) -> Result<Vec<HebDate>, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(file)?;
    let mut rdr = Reader::from_reader(file.as_bytes());
    let iter = rdr
        .deserialize()
        .map(|x: Result<ExcludedName, csv::Error>| x.unwrap());
    
    let mut filtered = dates;
    for row in iter {
        filtered.retain(|f| !f.title.contains(&row.names));
    }

    Ok(filtered)
}

#[derive(Debug)]
pub struct HebDate {
    pub title: String,
    pub date: NaiveDate,
}
impl HebDate {
    pub fn from(raw: &HebDateRaw) -> Self {
        Self {
            title: raw.title.clone(),
            date: NaiveDate::parse_from_str(&raw.date, "%Y-%m-%d")
                .expect("Failed to read Date from heb_dates file"),
        }
    }
}
impl Deref for HebDate {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HebDateRaw {
    pub title: String,
    pub date: String,
}
impl HebDateRaw {
    pub fn from(heb_date: &HebDate) -> Self {
        Self {
            title: heb_date.title.clone(),
            date: format!("{}", heb_date.date.format("%Y-%m-%d")),
        }
    }
}
#[derive(Deserialize)]
struct ExcludedName{
    pub names:String,
}
// #[cfg(test)]
// mod tests {
//     use chrono::NaiveDate;

//     use crate::{generate_heb, heb_cal::HebDate, EXCLUDED_DATES_PATH};

//     use super::exclued_holidays_from_file;

//     #[test]
//     fn exclude_date() {
//         let excluded_hebcal = r#"{
//     "Excluded Holidays": [
//         {
//             "title": "Purim"
//         },
//     ]
// }"#;
//         let hebcal = vec![HebDate {
//             title: "Purim".to_string(),
//             date: NaiveDate::from_ymd(***REMOVED***, 1, 1),
//         }];
//         let dates = generate_heb().expect("Error generating hebcal");
//         assert!(dates.iter().any(|f| f.title.contains("Purim")));
//         let filtered =
//             exclued_holidays_from_file(dates, &format!(".{}",EXCLUDED_DATES_PATH)).expect("Error filtering hebcal");
//         assert!(!filtered.iter().any(|f| f.title.contains("Purim")));
//     }
// }
