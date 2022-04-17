use std::result;

use chrono::NaiveDate;

use serde_json::{self, Value,from_str};

use crate::list::Soldier;
async fn get_heb_cal()->Result<String, reqwest::Error>{
    let url = "https://www.hebcal.com/hebcal?v=1&cfg=json&year=now&month=x&maj=on&mod=on&i=on&geo=none&c=off";
    let str = reqwest::get(url)?.text()?;
    Ok(str)
    
}


pub fn generate_heb()->Result<Vec<HebDate>,Box<dyn std::error::Error>>{
    let heb_cal = futures::executor::block_on(get_heb_cal())?;
    std::fs::write("./config/heb_date.json", &heb_cal)?;
    
    Ok(get_struct(&heb_cal)?)
    
}

fn get_struct(json:&str)->Result<Vec<HebDate>,Box<dyn std::error::Error>>{
    let data = r#json;
    let wrapper:Value=serde_json::from_str(&data)?;
    let wrapper = wrapper["items"].clone(); 
    let mut items = Vec::<HebDate>::new();
    let array = wrapper.as_array().unwrap();
    for item in array{
        let a = item.as_object().unwrap();
        items.push(HebDate{
            category:a["category"].as_str().unwrap().to_string(),
            date:NaiveDate::parse_from_str(a["date"].as_str().unwrap(), "%Y-%m-%d")?,
            subcat:a["subcat"].as_str().unwrap().to_string(),
            title:a["title"].as_str().unwrap().to_string(),
        });
    };
    Ok(items)
}

pub fn exclued_holidays_from_file(dates:Vec<HebDate>, file:&str)-> Result<Vec<HebDate>,Box<dyn std::error::Error>>{
    let file = std::fs::read_to_string(file)?;
    let json:Value = from_str(&file)?;
    let json:Value = json.as_object().expect("Could not find \"Excluded Holidays\"")["Excluded Holidays"];
    let holidays = Vec::<String>::new();
    for holiday in json.as_array().expect("Could not find holidays array").iter(){
        holidays.push(holiday.as_object().expect("holiday list reading error")["title"].as_str().expect("holiday needs to be a string").to_string())
    }
    let result = holidays.into_iter().filter(|x| x.title)

}

#[derive(Debug)]
pub struct HebDate{
    title:String,
    date:NaiveDate,
    category:String,
    subcat:String,
}