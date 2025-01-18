use reqwest::{Client, Response};
use serde_json::{json, Value};
use std::env;
use std::error::Error;
use crate::srt_parser::SRTProcessor;
use futures::stream::{FuturesUnordered, StreamExt};


#[derive(Clone)]
pub struct Translator {
    pub language: String,
    pub text: String,
    pub model: String,
    pub temperature: f32,
    pub max_token: u64,
}

impl Translator {
    pub async fn translate_openai(&self, subtitles: String) -> Result<String, Box<dyn Error>> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable is missing")?;
        let url: &str = "https://api.openai.com/v1/chat/completions";

        let client: Client = Client::new();
        let req: Value = json!({
            "model": self.model, // "gpt-4" or "gpt-3.5-turbo"
            "messages": [
                {"role": "system", "content": "You are an expert translator who speaks every language and love translating and helping others"},
                {"role": "user", "content": format!(
                    "Here is a list of dialog from a srt files, translate them in {}, in the input that
                    I give you, everytime there is an empty line it means that it's a different frame,keep it the same for the translation.
                    Just give me the raw translation, no comment inbetween or before. Thank you in advance\n{}",self.language,subtitles)}
            ],
            "max_tokens": self.max_token,
            "temperature": self.temperature,
        });

        let response: Response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&req)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            if let Some(content) = response_json
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice["message"]["content"].as_str())
            {
                Ok(content.to_string())
            } else {
                Err("Unexpected JSON structure".into())
            }
        } else {
            Err("Invalid response, status code is not 200".into())
        }
    }


}






pub async fn translate_single_processor(translator : Translator, processor : SRTProcessor) -> Result<SRTProcessor,Box<dyn Error>> {
        
    let str_text: String = processor.text_to_string();

    match translator.translate_openai(str_text).await {
        Ok(translated_text) => {
            println!("Translation successful: {}", translated_text);
            
            let parser = SRTProcessor::new();
            // Update the subtitles and write to a new file
            match parser.parse_llm_reponse(translated_text) {
                Ok(new_parser) => {
                    let update_result = new_parser.write("translation.srt".to_string());

                    match update_result {
                        Ok(()) => {println!("Translation saved to 'translation.srt'");}
                        Err(_e) => {
                            println!("Failed to save the new srt file");
                        }    
                    }
                }
                Err(e) => {
                    println!("Failed to process translated response: {}", e);
                    return Err(e.into());
                }
            }
            Ok(parser)
        }
        Err(e) => {
            eprintln!("An error occurred while calling OpenAI API: {}", e);
            return Err(e.into());
        }
    }
    
}


pub async fn translate_processor(
    translator : Translator,
    processor: SRTProcessor,
) -> Result<SRTProcessor, Box<dyn Error>> {
    let mut futures = FuturesUnordered::new();
    
    let processors = processor.split_by_token_size(translator.max_token as usize);
    for processor in processors {
        let future = translate_single_processor(translator.clone(),processor);
        futures.push(future);
    }
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        match result {
            Ok(translated_processor) => {
                results.push(translated_processor);
            }
            Err(e) => {
                eprintln!("Translation failed for a processor: {}", e);
                return Err(e);
            }
        }
    }
    Ok(SRTProcessor::from_concat(results))
}