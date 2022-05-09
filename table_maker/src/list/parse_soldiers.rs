use csv::Reader;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soldier {
    pub name: String,
    pub phone: String,
}

pub fn parse_candidates_from_file(file: &str)->Result<Vec<Soldier>, Box<dyn std::error::Error>>{
    let data = std::fs::read_to_string(file)?;
    let mut people = Vec::new();
    let mut rdr = Reader::from_reader(data.as_bytes());
    let iter = rdr
            .deserialize()
            .map(|x: Result<Soldier, csv::Error>| x.unwrap());
    for row in iter{
        people.push(row);
    }
    Ok(people)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_soldiers() {
        let s = r#"name,phone
amichai,***REMOVED***"#;
        std::fs::write("./test.csv", s).expect("Failed to write to test file");
        let vec = vec![Soldier {
            name: "amichai".to_string(),
            phone: "***REMOVED***".to_string(),
        }];
        let parsed = parse_candidates_from_file("./test.csv").expect("failed test");
        assert_eq!(vec.len(), parsed.len());
        assert_eq!(vec[0].name, parsed[0].name);
        assert_eq!(vec[0].phone, parsed[0].phone);
        std::fs::remove_file("./test").expect("Could nout remove file");
    }
}
