use super::Encoder;
use crate::flowgger::config::Config;
use crate::flowgger::record::{Record, SDValue};
use serde_json;
use serde_json::builder::ObjectBuilder;
use serde_json::value::Value;

#[derive(Clone)]
pub struct GelfEncoder {
    default_message: String,
    extra: Vec<(String, String)>,
}

impl GelfEncoder {
    pub fn new(config: &Config) -> GelfEncoder {
        let extra = match config.lookup("output.gelf_extra") {
            None => Vec::new(),
            Some(extra) => extra
                .as_table()
                .expect("output.gelf_extra must be a list of key/value pairs")
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.to_owned(),
                        v.as_str()
                            .expect("output.gelf_extra values must be strings")
                            .to_owned(),
                    )
                }).collect(),
        };
        let default_message = match config.lookup("output.gelf_default_message") {
            None => "-".to_string(),
            Some(setting) => setting
                .as_str().expect("output.gelf_default_message must be a string")
                .to_owned()
        };
        GelfEncoder { default_message, extra }
    }
}

impl Encoder for GelfEncoder {
    fn encode(&self, record: Record) -> Result<Vec<u8>, &'static str> {
        let mut map = ObjectBuilder::new()
            .insert("version".to_owned(), Value::String("1.1".to_owned()))
            .insert(
                "host".to_owned(),
                Value::String(if record.hostname.is_empty() { "unknown".to_owned() } else { record.hostname }),
            )
            .insert(
                "short_message".to_owned(),
                Value::String(record.msg.unwrap_or(self.default_message.to_owned())),
            )
            .insert("timestamp".to_owned(), Value::F64(record.ts));
        if let Some(severity) = record.severity {
            map = map.insert("level".to_owned(), Value::U64(u64::from(severity)));
        }
        if let Some(full_msg) = record.full_msg {
            map = map.insert("full_message".to_owned(), Value::String(full_msg));
        }
        if let Some(appname) = record.appname {
            map = map.insert("application_name".to_owned(), Value::String(appname));
        }
        if let Some(procid) = record.procid {
            map = map.insert("process_id".to_owned(), Value::String(procid));
        }
        for (name, value) in self.extra.iter().cloned() {
            map = map.insert(name, Value::String(value));
        }
        if let Some(sd) = record.sd {
            if let Some(sd_id) = sd.sd_id {
                map = map.insert("sd_id".to_owned(), Value::String(sd_id));
            }
            for (name, value) in sd.pairs {
                let value = match value {
                    SDValue::String(value) => Value::String(value),
                    SDValue::Bool(value) => Value::Bool(value),
                    SDValue::F64(value) => Value::F64(value),
                    SDValue::I64(value) => Value::I64(value),
                    SDValue::U64(value) => Value::U64(value),
                    SDValue::Null => Value::Null,
                };
                map = map.insert(name, value);
            }
        }
        let json = serde_json::to_vec(&map.build()).or(Err("Unable to serialize to JSON"))?;
        Ok(json)
    }
}
