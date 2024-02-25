pub mod received;
pub mod sent;

mod date_serde {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y.%m.%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[repr(transparent)]
    struct TestStruct(#[serde(with = "date_serde")] NaiveDate);

    #[test]
    fn date_serde_works() {
        let date = TestStruct(NaiveDate::from_ymd_opt(2021, 1, 30).unwrap());
        let s = serde_json::to_string(&date).unwrap();
        assert_eq!(s, "\"2021.01.30\"");

        let parsed_date: TestStruct = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed_date, date);
    }
}
