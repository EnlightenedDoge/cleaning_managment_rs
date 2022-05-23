pub mod table {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use csv::{self, Reader, Writer};
    use table_maker::{HebDateRaw, NamesTableRaw, Soldier};
    use table_configs::paths;


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
    ) -> Result<Vec<table_maker::HebDate>, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(&paths::get_output_path("excluded_dates.csv"))?;
        let mut rdr = Reader::from_reader(file.as_bytes());
        let iter = rdr
            .deserialize()
            .map(|x: Result<HebDateRaw, csv::Error>| x.unwrap());
        Ok(iter.map(|x| table_maker::HebDate::from(&x)).collect())
    }
}
