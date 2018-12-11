use super::Splitter;
use capnp;
use capnp::message::ReaderOptions;
use crate::flowgger::decoder::Decoder;
use crate::flowgger::encoder::Encoder;
use crate::flowgger::record::{Record, SDValue, StructuredData, FACILITY_MAX, SEVERITY_MAX};
use crate::record_capnp;
use std::io::{BufReader, Read};
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;

pub struct CapnpSplitter;

impl<T: Read> Splitter<T> for CapnpSplitter {
    fn run(
        &self,
        buf_reader: BufReader<T>,
        tx: SyncSender<Vec<u8>>,
        _decoder: Box<Decoder>,
        encoder: Box<Encoder>,
    ) {
        let mut buf_reader = buf_reader;
        loop {
            let message_reader =
                match capnp::serialize::read_message(&mut buf_reader, ReaderOptions::new()) {
                    Err(e) => match e.kind {
                        capnp::ErrorKind::Failed | capnp::ErrorKind::Unimplemented => {
                            error!("Capnp decoding error: {}", e.description);
                            return;
                        }
                        capnp::ErrorKind::Overloaded => {
                            thread::sleep(Duration::from_millis(250));
                            continue;
                        }
                        capnp::ErrorKind::Disconnected => {
                            warn!("Client hasn't sent any data for a while - Closing idle connection");
                            return;
                        }
                    },
                    Ok(message_reader) => message_reader,
                };
            let message: record_capnp::record::Reader = message_reader.get_root().unwrap();
            let record = match handle_message(message) {
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
                Ok(record) => record,
            };
            match encoder.encode(record) {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(reencoded) => tx.send(reencoded).unwrap(),
            };
        }
    }
}

fn get_pairs(
    message_pairs: Option<capnp::struct_list::Reader<record_capnp::pair::Owned>>,
    message_extra: Option<capnp::struct_list::Reader<record_capnp::pair::Owned>>,
) -> Vec<(String, SDValue)> {
    let pairs_count = message_pairs
        .and_then(|x| Some(x.len()))
        .or(Some(0))
        .unwrap() as usize
        + message_extra
        .and_then(|x| Some(x.len()))
        .or(Some(0))
        .unwrap() as usize;
    let mut pairs = Vec::with_capacity(pairs_count);
    if let Some(message_pairs) = message_pairs {
        for message_pair in message_pairs.iter() {
            let name = match message_pair.get_key() {
                Ok(name) => if name.starts_with('_') {
                    name.to_owned()
                } else {
                    format!("_{}", name)
                },
                _ => continue,
            };
            let value = match message_pair.get_value().which() {
                Ok(record_capnp::pair::value::String(Ok(x))) => SDValue::String(x.to_owned()),
                Ok(record_capnp::pair::value::Bool(x)) => SDValue::Bool(x),
                Ok(record_capnp::pair::value::F64(x)) => SDValue::F64(x),
                Ok(record_capnp::pair::value::I64(x)) => SDValue::I64(x),
                Ok(record_capnp::pair::value::U64(x)) => SDValue::U64(x),
                Ok(record_capnp::pair::value::Null(())) => SDValue::Null,
                _ => continue,
            };
            pairs.push((name, value));
        }
    }
    if let Some(message_extra) = message_extra {
        for message_pair in message_extra.iter() {
            match (message_pair.get_key(), message_pair.get_value().which()) {
                (Ok(name), Ok(record_capnp::pair::value::String(Ok(value)))) => {
                    pairs.push((name.to_owned(), SDValue::String(value.to_owned())))
                }
                _ => continue,
            }
        }
    }
    pairs
}

fn get_sd(message: record_capnp::record::Reader) -> Result<Option<StructuredData>, &'static str> {
    let sd_id = message.get_sd_id().and_then(|x| Ok(x.to_owned())).ok();
    let pairs = message.get_pairs().ok();
    let extra = message.get_extra().ok();
    let pairs = if pairs.is_none() && extra.is_none() {
        if sd_id.is_none() {
            return Ok(None);
        }
        Vec::new()
    } else {
        get_pairs(pairs, extra)
    };
    Ok(Some(StructuredData {
        sd_id: sd_id,
        pairs: pairs,
    }))
}

fn handle_message(message: record_capnp::record::Reader) -> Result<Record, &'static str> {
    let ts = message.get_ts();
    if ts.is_nan() || ts <= 0.0 {
        return Err("Missing timestamp");
    }
    let hostname = message
        .get_hostname()
        .and_then(|x| Ok(x.to_owned()))
        .or(Err("Missing host name"))?;
    let facility = match message.get_facility() {
        facility if facility <= FACILITY_MAX => Some(facility),
        _ => None,
    };
    let severity = match message.get_severity() {
        severity if severity <= SEVERITY_MAX => Some(severity),
        _ => None,
    };
    let appname = message.get_appname().and_then(|x| Ok(x.to_owned())).ok();
    let procid = message.get_procid().and_then(|x| Ok(x.to_owned())).ok();
    let msgid = message.get_msgid().and_then(|x| Ok(x.to_owned())).ok();
    let msg = message.get_msg().and_then(|x| Ok(x.to_owned())).ok();
    let full_msg = message.get_full_msg().and_then(|x| Ok(x.to_owned())).ok();
    let sd = get_sd(message)?;
    Ok(Record {
        ts: ts,
        hostname: hostname,
        facility: facility,
        severity: severity,
        appname: appname,
        procid: procid,
        msgid: msgid,
        msg: msg,
        full_msg: full_msg,
        sd: sd,
    })
}
