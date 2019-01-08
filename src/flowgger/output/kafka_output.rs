use super::Output;
use crate::flowgger::config::Config;
use crate::flowgger::merger::Merger;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use rdkafka::ClientConfig;
use std::collections::BTreeMap;
use toml::Value;

const KAFKA_DEFAULT_THREADS: u32 = 1;

pub struct KafkaOutput {
    config: KafkaConfig,
    threads: u32,
}

#[derive(Clone)]
struct KafkaConfig {
    topic: String,
    librdkafka: BTreeMap<String, Value>,
}

struct KafkaWorker {
    arx: Arc<Mutex<Receiver<Vec<u8>>>>,
    producer: FutureProducer,
    config: KafkaConfig,
}

impl KafkaWorker {
    fn new(arx: Arc<Mutex<Receiver<Vec<u8>>>>, config: KafkaConfig) -> KafkaWorker {
        let mut client_config = ClientConfig::new();
        for (k, v) in config.librdkafka.iter() {
            client_config.set(k, v.as_str().expect("All output.librdkafka settings MUST be strings even numbers"));
        }
        let producer: FutureProducer = client_config
            .create()
            .expect("Producer creation error");

        KafkaWorker {
            arx: arx,
            producer: producer,
            config: config,
        }
    }

    fn run(&mut self) {
        loop {
            let bytes = match { self.arx.lock().unwrap().recv() } {
                Ok(line) => line,
                Err(_) => return,
            };
            self.producer.send::<Vec<u8>, Vec<u8>>(FutureRecord::to(&self.config.topic).payload(&bytes), -1);
        }
    }
}

impl KafkaOutput {
    pub fn new(config: &Config) -> KafkaOutput {
        let topic = config
            .lookup("output.topic").expect("output.topic must be a string")
            .as_str().expect("output.topic must be a string")
            .to_owned();
        let librdconfig = config
            .lookup("output.librdkafka").expect("output.librdkafka must be set")
            .as_table().expect("output.librdkafka must be set")
            .to_owned();
        let threads = config
            .lookup("output.kafka_threads")
            .map_or(KAFKA_DEFAULT_THREADS, |x| {
                x.as_integer()
                    .expect("output.kafka_threads must be a 32-bit integer") as u32
            });
        KafkaOutput {
            config: KafkaConfig { topic, librdkafka: librdconfig },
            threads,
        }
    }
}

impl Output for KafkaOutput {
    fn start(&self, arx: Arc<Mutex<Receiver<Vec<u8>>>>, merger: Option<Box<Merger>>) {
        if merger.is_some() {
            error!("Output framing is ignored with the Kafka output");
        }
        for id in 0..self.threads {
            let tarx = Arc::clone(&arx);
            let tconfig = self.config.clone();
            thread::Builder::new().name(format!("kafka-output-{}", id)).spawn(move || {
                let mut worker = KafkaWorker::new(tarx, tconfig);
                worker.run();
            }).unwrap();
        }
    }
}
