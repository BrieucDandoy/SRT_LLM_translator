use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead, BufReader, Read};
pub struct Subtitle {
    index : usize,
    start : String,
    end : String,
    text : String,
}
impl Subtitle {
    pub fn new(index : usize,start : String,end : String,text : String) -> Self {
        Self {index,start,end,text}
    }
}

impl ToString for Subtitle {
    fn to_string(&self) -> String {
        format!("{}\n{} --> {}\n{}\n\n",self.index,self.start,self.end,self.text)
    }
}





pub struct SRTProcessor {
    subtitles : Vec<Subtitle>,
    language : &str,
}


impl SRTProcessor {
    pub fn parse(&mut self,path : Path)  -> io::Result<&mut Self> {

        let mut file: File = File::open(path)?;
        let mut contents: String = String::new();
        let reader: BufReader<File> = BufReader::new(file);
        let mut line_iter: io::Lines<BufReader<File>> = reader.lines();
        while let Some(line) = line_iter.next() {
            line_content = line.unwrap()?;
            if line_content.parse::<U32>() {
                let times : Vec<String> = line_iter.next().ok_or(err).unwrap()?.split(" --> ").map(|s: &str| s.to_string()).collect();
                let mut text: String = line_iter.next().ok_or(err).unwrap()?;
                let text2: &str = line_iter.next().ok_or(err).unwrap()?.as_str();
                match text2 {
                    "\n" => line_iter.next(),
                    _ => text += text2
                }
                let subtitle : Subtitle = Subtitle::new(line_content, times[0], times[1], text);
                self.subtitles.append(subtitle);
            }
            else {_ = line_iter.next()}
        }
        Ok(self)
    }
}















