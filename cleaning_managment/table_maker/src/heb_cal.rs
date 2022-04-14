use chrono::NaiveDate;

use serde_json::{self, Value};
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

// #[derive(Deserialize,Serialize,Debug)]
// enum Value {
//     Null,
//     Bool(bool),
//     Number(i32),
//     String(String),
//     Array(Vec<Value>),
//     Object(HashMap<String, Value>),
// }

#[derive(Debug)]
pub struct HebDate{
    title:String,
    date:NaiveDate,
    category:String,
    subcat:String,
}