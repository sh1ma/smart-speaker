use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;
use serde::{de, Deserialize, Serialize};
use tracing::info;

#[async_trait]
#[automock]
pub trait KotonohaClient {
    async fn ask(&self, wav: Vec<u8>) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct KotonohaClientImpl {
    host: String,
    port: u16,
}

impl KotonohaClientImpl {
    pub fn new(host: String, port: u16) -> Self {
        KotonohaClientImpl { host, port }
    }

    async fn transcribe(&self, wav: Vec<u8>) -> Result<TranscribeOutput> {
        let endpoint = format!("http://{}:{}/transcribe", self.host, self.port);
        let client = reqwest::Client::new();
        let res = client.post(&endpoint).body(wav).send().await?;
        let output: TranscribeOutput = serde_json::from_str(&res.text().await?)?;
        Ok(output)
    }

    async fn think(&self, text: String) -> Result<ThinkOutput> {
        let endpoint = format!("http://{}:{}/talk", self.host, self.port);
        let client = reqwest::Client::new();
        let input = ThinkInput { text };
        let res = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&input)?)
            .send()
            .await?;

        let output: ThinkOutput = serde_json::from_str(&res.text().await?)?;
        Ok(output)
    }

    async fn synthesize(&self, text: String) -> Result<Vec<u8>> {
        let voicevox_host = format!("http://{}:50021", self.host);

        // audioqueryの生成
        let audio_query_input = AudioQueryInput {
            text: text.clone(),
            speaker: "3".to_string(),
        };

        let audio_query = reqwest::Client::new()
            .post(&format!("{}/audio_query", voicevox_host))
            .query(&audio_query_input)
            .send()
            .await?;

        let mut audio_query_output: serde_json::Value =
            serde_json::from_str(&audio_query.text().await?)?;

        info!("{:?}", &audio_query_output);

        audio_query_output["speedScale"] = serde_json::Value::String("1.3".to_string());

        let wav = reqwest::Client::new()
            .post(&format!("{}/synthesis", voicevox_host))
            .header("Content-Type", "application/json")
            .query(&SynthesisQuery {
                speaker: "3".to_string(),
            })
            .body(audio_query_output.to_string())
            .send()
            .await?;

        let output: Vec<u8> = wav.bytes().await?.to_vec();
        Ok(output)
    }
}

#[async_trait]
impl KotonohaClient for KotonohaClientImpl {
    async fn ask(&self, wav: Vec<u8>) -> Result<Vec<u8>> {
        let command_text = self.transcribe(wav).await?.text;
        let think_output = self.think(command_text).await?;
        let response = think_output.output;
        let synthesized_wav = self.synthesize(response).await?;
        Ok(synthesized_wav)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranscribeOutput {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThinkInput {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThinkOutput {
    pub output: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AudioQueryInput {
    pub text: String,
    pub speaker: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SynthesisQuery {
    pub speaker: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SynthesisInput {
    pub text: String,
}
