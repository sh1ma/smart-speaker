use std::{sync::Arc, thread::spawn};

use anyhow::Result;
use reqwest::blocking::get;
use std::io::Write;
use std::process::Command;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::info;

mod kotonoha_client;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();

    let config = AkaneConfig::from_env().unwrap();
    let akane = Akane::new(config);

    akane.run().await.unwrap();
}

struct Akane {
    config: AkaneConfig,
    kotonoha_client: Arc<dyn kotonoha_client::KotonohaClient + Sync + Send>,
}

impl Akane {
    fn new(config: AkaneConfig) -> Self {
        let kotonoha = Arc::new(kotonoha_client::KotonohaClientImpl::new(
            config.kotonoha_host.clone(),
            config.kotonoha_port,
        ));
        Akane {
            config,
            kotonoha_client: kotonoha,
        }
    }

    async fn run(&self) -> Result<()> {
        info!("Akane started");
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            self.config.julius_host, self.config.julius_port
        ))
        .await?;
        loop {
            let mut buf = Vec::with_capacity(1024);
            stream.read_buf(&mut buf).await?;

            let received = String::from_utf8(buf);
            match received {
                Ok(received) => {
                    let pattern = r###"WORD=\"([^"]+)"###;
                    let captures = regex::Regex::new(pattern)?.captures(&received);
                    match captures {
                        Some(captures) => {
                            let word = captures.get(1).unwrap().as_str();
                            if word == "おーけーうさみ" {
                                info!("triggered: {}", word);
                                info!("adinrec started");
                                let _ = Command::new("adinrec").arg("out.wav").output()?;
                                info!("adinrec finished");
                                info!("open out.wav");
                                let audio_file = std::fs::read("out.wav")?;

                                info!("ask");
                                let resp_wav = self.kotonoha_client.ask(audio_file).await?;
                                let aplay = Command::new("aplay")
                                    .arg("-D")
                                    .arg("plughw:2,0")
                                    .arg("-")
                                    .stdin(std::process::Stdio::piped())
                                    .spawn()?;
                                aplay.stdin.unwrap().write_all(&resp_wav)?;
                                let mut output_file = std::fs::File::create("prev.wav")?;
                                output_file.write_all(&resp_wav)?;
                            }
                        }
                        None => {
                            info!("received: {}", received);
                        }
                    }
                }
                Err(e) => {
                    info!("received: {}", e);
                }
            }
        }

        todo!()
    }
}

struct AkaneConfig {
    julius_host: String,
    julius_port: u16,
    kotonoha_host: String,
    kotonoha_port: u16,
}

impl AkaneConfig {
    fn from_env() -> Result<Self> {
        let julius_host = load_julius_host()?;
        let julius_port = load_julius_port()?;
        let kotonoha_host = load_kotonoha_host()?;
        let kotonoha_port = load_kotonoha_port()?;

        Ok(AkaneConfig {
            julius_host,
            julius_port,
            kotonoha_host,
            kotonoha_port,
        })
    }
}

fn load_julius_host() -> Result<String> {
    let host = load_env("JULIUS_HOST")?;
    Ok(host)
}

fn load_julius_port() -> Result<u16> {
    let port = load_env("JULIUS_PORT")?.parse::<u16>()?;
    Ok(port)
}

fn load_kotonoha_host() -> Result<String> {
    let host = load_env("KOTONOHA_HOST")?;
    Ok(host)
}

fn load_kotonoha_port() -> Result<u16> {
    let port = load_env("KOTONOHA_PORT")?.parse::<u16>()?;
    Ok(port)
}

fn load_env(env_name: &str) -> Result<String> {
    let env = std::env::var(env_name)?;
    Ok(env)
}
