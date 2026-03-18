use anyhow::{Error, Result};
use icu::segmenter::SentenceSegmenter;
use itertools::Itertools;

pub fn segment(text: Vec<String>) -> Result<Vec<Vec<String>>> {
    let segmenter = SentenceSegmenter::new(Default::default());
    let sentances = text
        .iter()
        .map(|s| {
            segmenter
                .segment_str(s)
                .tuple_windows()
                .map(|(i, j)| s[i..j].to_string())
                .collect()
        })
        .collect::<Vec<Vec<String>>>();
    Ok(sentances)
}
#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn init() {
        let text = "Hello world. This is Rust.";
        let sentances = segment(vec![text.to_string()]);
        sentances.iter().for_each(|s| {
            s.iter().for_each(|v| {
                let segments = v.iter().map(|a| a.as_str()).collect::<Vec<&str>>();
                assert_eq!(segments, &["Hello world. ", "This is Rust."]);
            });
        });
    }
}
