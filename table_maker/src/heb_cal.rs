use chrono::NaiveDate;

use serde_json::{self, from_str, Value};

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

pub fn exclued_holidays_from_file(
    dates: Vec<HebDate>,
    file: &str,
) -> Result<Vec<HebDate>, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(file)?;
    let json: Value = from_str(&file)?;
    let json: &Value = &json
        .as_object()
        .expect("Could not find \"Excluded Holidays\"")["Excluded Holidays"];
    let mut filtered = dates;
    for holiday in json
        .as_array()
        .expect("Could not find holidays array")
        .iter()
    {
        let title = holiday.as_object().expect("holiday list reading error")["title"]
            .as_str()
            .expect("holiday needs to be a string")
            .to_string();
        filtered.retain(|f| !f.title.contains(&title));
    }
    Ok(filtered)
}

#[derive(Debug)]
pub struct HebDate {
    pub title: String,
    pub date: NaiveDate,
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