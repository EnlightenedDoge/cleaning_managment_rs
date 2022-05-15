pub mod construction {
    use crate::heb_cal::exclude_holidays_from_file;
    use crate::heb_cal::generate_heb;
    use crate::list::*;
    use chrono::Datelike;
    use chrono::NaiveDate;
    use csv::Writer;
    use serde::Deserialize;
    use serde::Serialize;

    use crate::{CONFIG_PATH, EXCLUDED_DATES_PATH, SOLDIERS_PATH};

    pub fn create_table(exclude_dates: bool) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        let config = load_config()?;
        let mut heb_cal = generate_heb()?;
        if exclude_dates {
            heb_cal = exclude_holidays_from_file(heb_cal, EXCLUDED_DATES_PATH)?;
        }
        let mut soldiers = parse_candidates_from_file(SOLDIERS_PATH)?;
        soldiers.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        let dates = get_dates(soldiers, heb_cal, config.start_date, config.range);
        let mut wtr_raw = Writer::from_writer(vec![]);
        let mut wtr_beaut = Writer::from_writer(vec![]);
        
        let ser:Vec<Raw> = dates.iter().map(|x| Raw{date:x.date.to_string(),name:x.soldier.name.clone(),number:x.soldier.phone.clone()}).collect();
        write_csv(&format!("{}output_table.csv", config.output_path), &ser)?;
        for date in dates.iter() {
            wtr_beaut.serialize(BeautyRaw {
                name: date.soldier.name.clone(),
                date: date.date.to_string(),
                day: date.date.weekday().to_string(),
            })?;
        }
        std::fs::write(
            format!("{}output_table.csv", config.output_path),
            String::from_utf8(wtr_raw.into_inner()?)?,
        )?;
        std::fs::write(
            format!("{}beautified_table.csv", config.output_path),
            String::from_utf8(wtr_beaut.into_inner()?)?,
        )?;
        Ok(dates)
    }

    fn load_config() -> Result<ConfigMaker, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string(CONFIG_PATH)?;
        let config: ConfigRaw = serde_json::from_str(&config)?;
        let config = ConfigMaker::from(config);
        Ok(config)
    }

    fn write_csv<T>(file_path:&str, t:&Vec<T>)->Result<(),Box<dyn std::error::Error>>where T:Serialize{
        let mut table = Writer::from_writer(vec![]);
        for row in t {
            table.serialize(row)?;
        }
        std::fs::write(file_path, String::from_utf8(table.into_inner()?)?);
        Ok(())
    }

    #[derive(Deserialize)]
    pub struct ConfigRaw {
        pub start_date: String,
        pub range: usize,
        pub output_path: String,
        pub send_time: String,
        pub reset_time: String,
        pub maintainer: String,
        pub alert_day: String,
        pub weekend:Vec<u16>,
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
    pub struct BeautyRaw {
        pub day: String,
        pub date: String,
        pub name: String,
    }
}

pub mod modification {}
