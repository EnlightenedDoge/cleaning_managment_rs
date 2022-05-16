pub const CONFIG_PATH: &str = "./config/config.json";

pub mod config {

    use chrono::{NaiveDate, NaiveTime};
    use table_maker::ConfigRaw;
    pub fn load_config(config_path:&str) -> Result<ConfigReader, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string(config_path)?;
        let config: ConfigRaw = serde_json::from_str(&config)?;
        let config = ConfigReader::from(config);
        Ok(config)
    }

    #[derive(Clone)]
    pub struct ConfigReader {
        pub start_date: NaiveDate,
        pub range: usize,
        pub output_path: String,
        pub send_time: NaiveTime,
        pub reset_time: NaiveTime,
        pub maintainer: String,
        pub alert_day: chrono::Weekday,
        pub weekend: Vec<chrono::Weekday>,
    }
    impl ConfigReader {
        fn from(config: ConfigRaw) -> Self {
            Self {
                output_path: config.output_path,
                range: config.range,
                start_date: NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d").unwrap(),
                send_time: NaiveTime::parse_from_str(&config.send_time, "%H:%M:%S").unwrap(),
                reset_time: NaiveTime::parse_from_str(&config.reset_time, "%H:%M:%S").unwrap(),
                maintainer: config.maintainer,
                alert_day: int_to_weekday(config.alert_day.parse().unwrap()),
                weekend: config.weekend.iter().map(|x| int_to_weekday(*x)).collect(),
            }
        }
    }
    //sunday=1,saturday = 7
    fn int_to_weekday(i: usize) -> chrono::Weekday {
        use chrono::Weekday;
        let weekday = [
            Weekday::Sun,
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
        ];
        if let Some(day) = weekday.get(i - 1) {
            return day.clone();
        }
        panic!("weekday config has wrong number. sun = 1.");
    }
}

pub mod table {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use csv::{self, Reader, Writer};
    use table_maker::{HebDateRaw, NamesTableRaw, Soldier};

    use super::config::ConfigReader;

    pub fn get_soldiers_table(
        filepath: &str,
    ) -> Result<HashMap<NaiveDate, Soldier>, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(&filepath)?;
        let mut map = HashMap::<NaiveDate, Soldier>::new();

        let mut rdr = Reader::from_reader(file.as_bytes());
        let iter = rdr
            .deserialize()
            .map(|x: Result<NamesTableRaw, csv::Error>| x.unwrap());

        for row in iter {
            map.insert(
                NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").unwrap(),
                Soldier {
                    name: row.name,
                    phone: row.number,
                },
            );
        }
        Ok(map)
    }
    pub fn update_soldiers_table(
        filepath: &str,
        table: &HashMap<NaiveDate, Soldier>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        let mut rows = Vec::<NamesTableRaw>::new();
        for p in table {
            rows.push(NamesTableRaw {
                date: String::from(p.0.format("%Y-%m-%d").to_string()),
                name: String::from(&p.1.name),
                number: String::from(&p.1.phone),
            });
        }
        rows.sort_by(|a, b| {
            NaiveDate::parse_from_str(&a.date, "%Y-%m-%d")
                .unwrap()
                .cmp(&NaiveDate::parse_from_str(&b.date, "%Y-%m-%d").unwrap())
        });
        for row in rows {
            wtr.serialize(row)?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        std::fs::write(filepath, data)?;
        Ok(())
    }
    pub fn get_excluded_dates(
        conf: &ConfigReader,
    ) -> Result<Vec<table_maker::HebDate>, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(&format!("{}excluded_dates.csv", &conf.output_path))?;
        let mut rdr = Reader::from_reader(file.as_bytes());
        let iter = rdr
            .deserialize()
            .map(|x: Result<HebDateRaw, csv::Error>| x.unwrap());
        Ok(iter.map(|x| table_maker::HebDate::from(&x)).collect())
    }
}
