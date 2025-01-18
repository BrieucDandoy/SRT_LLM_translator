use crate::srt_parser;
use crate::translator;
use tokio;
use std::error::Error;


pub async fn translate(srt_path : String)->  Result<(), Box<dyn std::error::Error>> {
        let mut parser = srt_parser::SRTProcessor::new();

        if let Err(e) = parser.parse(srt_path.clone()) {
            eprintln!("An error occurred during parsing: {}", e);
            return Err(e.into());
        }

        println!("Parse is successful");
        let str_text: String = parser.text_to_string();

        let translator = translator::Translator {
            language: "French".to_string(),
            text: str_text.clone(), // Pass the text to translate
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.68,
            max_token: 1000,
        };

        let new_sub = translator::translate_processor(translator,parser).await;
        match new_sub {
            Ok(proc) => {
                let translated_file_name: String = format!("translated_{}",srt_path);
                let _ = proc.write(translated_file_name.clone());
                println!("File saved as {}",translated_file_name);
            }
            Err(e) => {
                return Err(e.into())
            }
        }
    Ok(())
    }