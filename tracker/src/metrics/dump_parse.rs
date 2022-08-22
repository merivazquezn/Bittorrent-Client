use bittorrent_rustico::bencode::{decode, encode, BencodeDecodedValue, BencodeDecoderError};
use chrono::offset::LocalResult;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::io;

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

            let from: NaiveDateTime = timestamp_string
                .parse()
                .map_err(|_| MetricsDumpError::ParseError)?;

            let datetime_result: LocalResult<DateTime<Local>> = Local.from_local_datetime(&from);
            let timestamp: DateTime<Local> = match datetime_result {
                LocalResult::None => {
                    return Err(MetricsDumpError::ParseError);
                }
                LocalResult::Single(datetime) => datetime,
                LocalResult::Ambiguous(datetime, _) => datetime,
            };

            stat_timestamp_list.push((stat, timestamp));
        }
        result.insert(key, stat_timestamp_list);
    }

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
