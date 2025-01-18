use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error as IOError, Write};


#[derive(Clone)]
pub struct Subtitle {
    index: usize,
    start: String,
    end: String,
    text: String,
}


impl Subtitle {
    pub fn new(index: usize, start: String, end: String, text: String) -> Self {
        Self {
            index,
            start,
            end,
            text,
        }
    }
}

impl ToString for Subtitle {
    fn to_string(&self) -> String {
        format!(
            "{}\n{} --> {}\n{}\n",
            self.index, self.start, self.end, self.text
        )
    }
}
impl Subtitle {
    fn text_to_string(&self) -> String {
        format!("\n{}", self.text)
    }

    fn get_token_number(&self) -> usize {
        self.text.split_whitespace().count()
    }
}
#[derive(Clone)]
pub struct SRTProcessor {
    pub subtitles: Vec<Subtitle>,
    pub language: Option<String>,
}

impl SRTProcessor {
    pub fn parse(&mut self, path: String) -> io::Result<&mut Self> {
        let file: File = File::open(path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let mut line_iter: io::Lines<BufReader<File>> = reader.lines();
        while let Some(line) = line_iter.next() {
            let line_content: String = match line {
                Ok(line_no_error) => line_no_error,
                Err(err) => return Err(err),
            };
            if line_content.parse::<usize>().is_ok() {
                let times_line = line_iter.next();
                let times: Vec<String> = match times_line {
                    Some(some_line) => match some_line {
                        Err(err) => return Err(err),
                        Ok(valid_time_line) => valid_time_line,
                    },
                    None => {
                        return Ok(self);
                    }
                }
                .split(" --> ")
                .map(|s| s.to_string())
                .collect();

                if times.len() != 2 {
                    return Err(IOError::new(
                        io::ErrorKind::InvalidData,
                        format!("Expected times vec length of 2, found {}", times.len()),
                    ));
                }

                let text_line = line_iter.next();
                let mut text: String = match text_line {
                    Some(text_line) => match text_line {
                        Ok(text_line) => text_line,
                        Err(e) => return Err(e),
                    },
                    None => return Ok(self),
                };
                let text2 = line_iter.next();
                let text2_string = match text2 {
                    Some(text2_res) => match text2_res {
                        Ok(text2_string) => text2_string,
                        Err(e) => return Err(e),
                    },
                    None => return Ok(self),
                };

                text = match text2_string.as_str() {
                    "\n" => {
                        line_iter.next();
                        text
                    }
                    _ => text + "\n" + text2_string.as_str() + "\n",
                };

                let subtitle: Subtitle = Subtitle::new(
                    // we can use unwrap because we know it is ok from the if condition of the block
                    line_content.parse::<usize>().unwrap(),
                    times[0].clone(),
                    times[1].clone(),
                    text,
                );
                self.subtitles.push(subtitle);
            }
        }
        Ok(self)
    }

    pub fn new() -> Self {
        SRTProcessor {
            subtitles: Vec::new(),
            language: None,
        }
    }

    pub fn text_to_string(&self) -> String {
        self.subtitles
            .iter()
            .map(|sub| sub.text_to_string())
            .collect::<String>()
    }

    pub fn parse_llm_reponse(&self, response: String) -> Result<SRTProcessor, Box<dyn Error>> {
        let updated_subtitles: Vec<Subtitle> = response
            .split("\n\n")
            .zip(self.subtitles.iter())
            .map(|(s, sub)| {
                Subtitle::new(
                    sub.index.clone(),
                    sub.start.clone(),
                    sub.end.clone(),
                    s.to_string(),
                )
            })
            .collect();

        if updated_subtitles.len() != self.subtitles.len() {
            return Err(
                "Invalid response, differents length for translated version and original version"
                    .into(),
            );
        }

        Ok(SRTProcessor {
            subtitles: updated_subtitles,
            language: None,
        })
    }
    pub fn write(&self, path: String) -> io::Result<()> {
        let mut file = File::create(path)?;
        for sub in self.subtitles.iter() {
            file.write_all(sub.to_string().as_bytes())?;
        }
        Ok(())
    }

    pub fn split(&self,chunk_size : usize) -> Vec<SRTProcessor> {
        let mut output = Vec::new();
        for chunk in self.subtitles.chunks(chunk_size) {
            let chunk_srt_processor = SRTProcessor {
                language : self.language.clone(),
                subtitles : chunk.to_vec()
            };
            output.push(chunk_srt_processor);
        }
        output
    }


    pub fn split_by_token_size(&self, max_token_length: usize) -> Vec<SRTProcessor> {
        let mut output: Vec<SRTProcessor> = Vec::new();
        let mut token_count: usize = 0;
        let mut min_slice_idx: usize = 0;
    
        let total_subtitles = self.subtitles.len();
    
        for (idx, sub) in self.subtitles.iter().enumerate() {
            let sub_token_count = sub.get_token_number();
    
            if token_count + sub_token_count > max_token_length {
                output.push(SRTProcessor {
                    subtitles: self.subtitles[min_slice_idx..idx].to_vec(),
                    language: self.language.clone(),
                });
                min_slice_idx = idx;
                token_count = sub_token_count;
            } else {
                token_count += sub_token_count;
            }
        }
    
        if min_slice_idx < total_subtitles {
            output.push(SRTProcessor {
                subtitles: self.subtitles[min_slice_idx..].to_vec(),
                language: self.language.clone(),
            });
        }
    
        output
    }



    pub fn from_concat(processors : Vec<SRTProcessor>)  -> Self{
        // take the first item language
        let mut output = SRTProcessor::new();
        for proc in processors.iter() {
            output.subtitles.extend(proc.subtitles.clone());
            if output.language == None {
                output.language = proc.language.clone();
            }
        }
        output
    }
}
