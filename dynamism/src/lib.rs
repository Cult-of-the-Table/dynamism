pub mod db;
//pub mod embed;
pub mod pmap;
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
    pub fn bind<U, F>(mut self, desc: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> Pipeline<U>,
    {
        self.logs.push(desc.to_string());
        let mut next = f(self.value);
        self.logs.append(&mut next.logs);
        Pipeline {
            value: next.value,
            logs: self.logs,
        }
    }

    pub fn map<U, F>(self, desc: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        self.bind(desc, |val| Pipeline::inject(f(val)))
    }

    pub async fn bind_async<U, F, Fut>(mut self, desc: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Pipeline<U>>,
    {
        self.logs.push(desc.to_string());
        let mut next = f(self.value).await;
        self.logs.append(&mut next.logs);
        Pipeline {
            value: next.value,
            logs: self.logs,
        }
    }
    pub async fn map_async<U, F, Fut>(self, desc: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = U>,
    {
        self.bind_async(desc, |val| async { Pipeline::inject(f(val).await) })
            .await
    }
}
