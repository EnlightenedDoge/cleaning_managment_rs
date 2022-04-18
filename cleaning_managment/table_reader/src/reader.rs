const CONFIG_PATH: &str = "./config/config.json";

pub mod config {
    use chrono::NaiveDate;
    use serde::Deserialize;
    pub fn load_config() -> Result<ConfigReader, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string(super::CONFIG_PATH)?;
        let config: ConfigRaw = serde_json::from_str(&config)?;
        let config = ConfigReader::from(config);
        Ok(config)
    }

    #[derive(Deserialize)]
    struct ConfigRaw {
        #[allow(dead_code)]
        start_date: String,
        #[allow(dead_code)]
        range: usize,
        output_path: String,
    #[allow(dead_code)]
    send_time: String,
    #[allow(dead_code)]
    reset_time: String,
    }

    pub struct ConfigReader {
        pub start_date: NaiveDate,
        pub range: usize,
        pub output_path: String,
    }
    impl ConfigReader {
        fn from(config: ConfigRaw) -> Self {
            Self {
                output_path: config.output_path,
                range: config.range,
                start_date: NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d").unwrap(),
            }
        }
    }
}

pub mod table {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use csv::{self, Reader};
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Raw {
        name: String,
        number: String,
        date: String,
    }

    pub fn get_soldiers_table(
        filepath: &str,
    ) -> Result<HashMap<NaiveDate, (String, String)>, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(&filepath)?;
        let mut map = HashMap::<NaiveDate, (String, String)>::new();

        let mut rdr = Reader::from_reader(file.as_bytes());
        let iter = rdr
            .deserialize()
            .map(|x: Result<Raw, csv::Error>| x.unwrap());

        for row in iter {
            map.insert(
                NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").unwrap(),
                (row.name, row.number),
            );
        }
        Ok(map)
    }
}
