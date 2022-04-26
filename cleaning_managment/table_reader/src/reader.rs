const CONFIG_PATH: &str = "./config/config.json";

pub mod config {
    use table_maker::ConfigRaw;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    pub fn load_config() -> Result<ConfigReader, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string(super::CONFIG_PATH)?;
        let config: ConfigRaw = serde_json::from_str(&config)?;
        let config = ConfigReader::from(config);
        Ok(config)
    }

    pub struct ConfigReader {
        pub start_date: NaiveDate,
        pub range: usize,
        pub output_path: String,
        pub send_time: NaiveTime,
        pub reset_time:NaiveTime,
    }
    impl ConfigReader {
        fn from(config: ConfigRaw) -> Self {
            Self {
                output_path: config.output_path,
                range: config.range,
                start_date: NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d").unwrap(),
                send_time:NaiveTime::parse_from_str(&config.send_time, "%H:%M:%S").unwrap(),
                reset_time:NaiveTime::parse_from_str(&config.reset_time, "%H:%M:%S").unwrap(),
            }
        }
    }
}

pub mod table {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use csv::{self, Reader};
    use table_maker::{Raw,Soldier};

    pub fn get_soldiers_table(
        filepath: &str,
    ) -> Result<HashMap<NaiveDate, Soldier>, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(&filepath)?;
        let mut map = HashMap::<NaiveDate, Soldier>::new();

        let mut rdr = Reader::from_reader(file.as_bytes());
        let iter = rdr
            .deserialize()
            .map(|x: Result<Raw, csv::Error>| x.unwrap());

        for row in iter {
            map.insert(
                NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").unwrap(),
                Soldier{name:row.name, phone:row.number},
            );
        }
        Ok(map)
    }
}
