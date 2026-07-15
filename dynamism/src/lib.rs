pub mod db;
pub mod embed;
pub mod reqwest;
pub mod scraper;
pub mod segmentation;
pub mod segmenter;
pub mod telemetry;
pub mod umap;
pub mod websearch;

pub struct Pipeline<T> {
    pub value: T,
    pub logs: Vec<String>,
}

impl<T> Pipeline<T> {
    pub fn inject(value: T) -> Self {
        Self {
            value,
            logs: Vec::new(),
        }
    }

    pub fn run_with_logs<U, F>(mut self, desc: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        self.logs.push(desc.to_string());
        Pipeline {
            value: f(self.value),
            logs: self.logs,
        }
    }
}
