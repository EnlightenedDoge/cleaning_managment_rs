pub mod construction {
    use crate::heb_cal::exclude_holidays_from_file;
    use crate::heb_cal::generate_heb;
    use crate::heb_cal::HebDateRaw;
    use crate::list::*;
    use chrono::Datelike;
    use csv::Writer;
    use serde::Deserialize;
    use serde::Serialize;
    use table_configs::{config, paths};

    pub fn create_table(exclude_dates: bool) -> Result<String, Box<dyn std::error::Error>> {
        let config = config::load_config();
        let mut heb_cal = generate_heb()?;
        if exclude_dates {
            heb_cal = exclude_holidays_from_file(heb_cal, &paths::get_excluded_holidays_path())?;
        }
        let mut people = parse_candidates_from_file(&paths::get_names_path());
        people.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        let dates = get_dates(&people, &heb_cal, &config.start_date, config.range);

        //create tables to ./output/
        //Names table to be used by program
        let raws: Vec<NamesTableRaw> = dates
            .iter()
            .map(|x| NamesTableRaw {
                date: x.date.to_string(),
                name: x.person.name.clone(),
                number: x.person.phone.clone(),
            })
            .collect();
        write_csv(&paths::get_output_path(&config.output_file_name), &raws)?;

        //Names table created for end-user use
        let raws_beaut: Vec<BeautyNameTableRaw> = dates
            .iter()
            .map(|x| BeautyNameTableRaw {
                date: x.date.to_string(),
                day: x.date.weekday().to_string(),
                name: x.person.name.clone(),
            })
            .collect();
        write_csv(&paths::get_output_path("beautified_table.csv"), &raws_beaut)?;
        //final excluded dates to be used by program as well
        let final_excluded_dates: Vec<HebDateRaw> =
            heb_cal.iter().map(|x| HebDateRaw::from(x)).collect();
        write_csv(
            &paths::get_output_path("excluded_dates.csv"),
            &final_excluded_dates,
        )?;

        Ok(std::fs::read_to_string(&paths::get_output_path(
            &config.output_file_name,
        ))?)
    }

    fn write_csv<T>(file_path: &str, t: &Vec<T>) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let mut table = Writer::from_writer(vec![]);
        for row in t {
            table.serialize(row).expect("Failed to serialize table");
        }
        std::fs::write(file_path, String::from_utf8(table.into_inner()?)?)?;
        Ok(())
    }

    #[derive(Serialize, Deserialize)]
    pub struct NamesTableRaw {
        pub name: String,
        pub number: String,
        pub date: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct BeautyNameTableRaw {
        pub day: String,
        pub date: String,
        pub name: String,
    }
}

pub mod modification {}
