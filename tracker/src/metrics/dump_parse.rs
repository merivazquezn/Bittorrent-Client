use bittorrent_rustico::bencode::{decode, encode, BencodeDecodedValue, BencodeDecoderError};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub enum MetricsDumpError {
    Io(io::Error),
    Bencode(BencodeDecoderError),
    FromUtf8(std::string::FromUtf8Error),
    TryFromInt(std::num::TryFromIntError),
    ParseError,
}

impl From<std::num::TryFromIntError> for MetricsDumpError {
    fn from(e: std::num::TryFromIntError) -> Self {
        MetricsDumpError::TryFromInt(e)
    }
}

impl From<io::Error> for MetricsDumpError {
    fn from(error: io::Error) -> Self {
        MetricsDumpError::Io(error)
    }
}

impl From<BencodeDecoderError> for MetricsDumpError {
    fn from(error: BencodeDecoderError) -> Self {
        MetricsDumpError::Bencode(error)
    }
}

impl From<std::string::FromUtf8Error> for MetricsDumpError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        MetricsDumpError::FromUtf8(error)
    }
}

fn fill_in_missing_timestamps(record: &mut HashMap<String, Vec<(i32, DateTime<Local>)>>) {
    for (_key, record_vector) in record.iter_mut() {
        if record_vector.is_empty() {
            println!("empty record vector, this shouldn't happen...");
            continue;
        }

        let record_clone = record_vector.clone();
        let last_entry = record_clone.last().unwrap();
        let time_to_fill = Local::now()
            .signed_duration_since(last_entry.1)
            .num_minutes();

        for i in 0..time_to_fill {
            record_vector.push((0, last_entry.1 + chrono::Duration::minutes(1 + i)));
        }
    }
}
pub fn get_encoded_record(record: HashMap<String, Vec<(i32, DateTime<Local>)>>) -> Vec<u8> {
    let mut hashmap: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();
    for (key, value) in record.iter() {
        let encoded_key: Vec<u8> = key.as_bytes().to_vec();

        let mut bencoded_list: Vec<BencodeDecodedValue> = Vec::new();
        for (stat, timestamp) in value.iter() {
            let bencoded_index: BencodeDecodedValue = BencodeDecodedValue::Integer((*stat).into());
            let bencoded_datetime: BencodeDecodedValue =
                BencodeDecodedValue::String(timestamp.to_string().as_bytes().to_vec());

            let stat_timestamp_list: Vec<BencodeDecodedValue> =
                vec![bencoded_index, bencoded_datetime];

            let bencoded_stat_timestamp_list: BencodeDecodedValue =
                BencodeDecodedValue::List(stat_timestamp_list.clone());

            bencoded_list.push(bencoded_stat_timestamp_list);
        }
        let bencoded_hashmap_value: BencodeDecodedValue = BencodeDecodedValue::List(bencoded_list);

        hashmap.insert(encoded_key, bencoded_hashmap_value);
    }

    let bencoded_hashmap: BencodeDecodedValue = BencodeDecodedValue::Dictionary(hashmap);
    encode(&bencoded_hashmap)
}

type TimeSeriesValue = (i32, DateTime<Local>);

pub fn get_dump_record(
    dump_path: &str,
) -> Result<HashMap<String, Vec<TimeSeriesValue>>, MetricsDumpError> {
    let content: Vec<u8> = std::fs::read(dump_path)?;
    let bencoded: BencodeDecodedValue = decode(&content)?;
    let bencode_dump: &HashMap<Vec<u8>, BencodeDecodedValue> = bencoded.get_as_dictionary()?;

    let mut result: HashMap<String, Vec<(i32, DateTime<Local>)>> = HashMap::new();
    for (key, value) in bencode_dump.iter() {
        let key: String = String::from_utf8(key.clone())?;
        let value: &Vec<BencodeDecodedValue> = value.get_as_list()?;
        let mut stat_timestamp_list: Vec<(i32, DateTime<Local>)> = Vec::new();
        for stat_timestamp in value.iter() {
            let stat: i32 = (*stat_timestamp.get_as_list()?[0].get_as_integer()?).try_into()?;

            let timestamp_string: String =
                String::from_utf8(stat_timestamp.get_as_list()?[1].get_as_string()?.clone())?;

            let timestamp: DateTime<Local> = DateTime::<Local>::from_str(&timestamp_string)
                .map_err(|_| MetricsDumpError::ParseError)?;
            stat_timestamp_list.push((stat, timestamp));
        }

        result.insert(key, stat_timestamp_list);
    }
    fill_in_missing_timestamps(&mut result);
    Ok(result)
}

impl std::fmt::Display for MetricsDumpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MetricsDumpError::Io(error) => write!(f, "IO error: {}", error),
            MetricsDumpError::Bencode(error) => write!(f, "Bencode error: {}", error),
            MetricsDumpError::FromUtf8(error) => write!(f, "FromUtf8 error: {}", error),
            MetricsDumpError::TryFromInt(error) => write!(f, "TryFromInt error: {}", error),
            MetricsDumpError::ParseError => write!(f, "Parse error"),
        }
    }
}
