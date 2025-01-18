mod srt_parser;
mod translator;
use dotenv::dotenv;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let srt_path: String = "example.srt".to_string();
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
            let _ = proc.write(format!("translated_{}",srt_path));
        }
        Err(e) => {
            return Err(e.into())
        }
    }

    Ok(())
}
