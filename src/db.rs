use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bytes::Bytes;
use tokio::task;

#[derive(Clone)]
pub struct DB {
    map: Arc<RwLock<HashMap<String, Bytes>>>,
    embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl DB {
    pub fn new() -> Self {
        DB {
            map: Arc::new(RwLock::new(HashMap::new())),
            embeddings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: Bytes) {
        let mut map = self.map.write().unwrap();
        map.insert(key.clone(), value.clone());

        // Check if value is valid UTF-8
        if let Ok(text) = std::str::from_utf8(&value) {
            let db_clone = self.clone();
            let key_clone = key.clone();
            let text_clone = text.to_string();
            tokio::spawn(async move {
                if let Ok(embedding) = task::spawn_blocking(move || {
                    crate::ai::get_embedding(&text_clone)
                }).await {
                    db_clone.add_vector(key_clone, embedding);
                } else {
                    eprintln!("Failed to generate embedding for key: {}", key_clone);
                }
            });
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let map = self.map.read().unwrap();
        map.get(key).cloned()
    }

    pub fn add_vector(&self, key: String, embedding: Vec<f32>) {
        let mut embeddings = self.embeddings.write().unwrap();
        embeddings.insert(key, embedding);
    }

    pub fn get_vector(&self, key: &str) -> Option<Vec<f32>> {
        let embeddings = self.embeddings.read().unwrap();
        embeddings.get(key).cloned()
    }

    pub fn find_similar(&self, query_vec: &[f32], k: usize) -> Vec<String> {
        let embeddings = self.embeddings.read().unwrap();
        let mut results: Vec<(f32, String)> = embeddings
            .iter()
            .map(|(key, vec)| {
                let score = crate::ai::cosine_similarity(vec, query_vec);
                (score, key.clone())
            })
            .collect();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        results.into_iter().take(k).map(|(_, key)| key).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_concurrent_set() {
        let db = Arc::new(DB::new());
        let mut handles = vec![];

        for i in 0..50 {
            let db_clone = Arc::clone(&db);
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let value = Bytes::from(format!("value_{}_{}", i, j));
                    db_clone.set(key, value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all values are set correctly
        let map = db.map.read().unwrap();
        assert_eq!(map.len(), 50 * 100);
        for i in 0..50 {
            for j in 0..100 {
                let key = format!("key_{}_{}", i, j);
                let value = map.get(&key).unwrap();
                assert_eq!(value, &Bytes::from(format!("value_{}_{}", i, j)));
            }
        }
    }

    #[tokio::test]
    async fn test_embedding_generation_for_utf8_value() {
        let db = Arc::new(DB::new());
        let key = "test_key".to_string();
        let value = Bytes::from("This is a test string for embedding.");
        db.set(key.clone(), value);

        // Wait for the background embedding generation
        sleep(Duration::from_secs(2)).await;

        let embedding = db.get_vector(&key);
        assert!(embedding.is_some());
        assert_eq!(embedding.unwrap().len(), 384);
    }

    #[tokio::test]
    async fn test_no_embedding_for_non_utf8_value() {
        let db = Arc::new(DB::new());
        let key = "test_key2".to_string();
        let value = Bytes::from(vec![0xff, 0xfe, 0xfd]); // Invalid UTF-8
        db.set(key.clone(), value);

        // Wait a bit
        sleep(Duration::from_millis(100)).await;

        let embedding = db.get_vector(&key);
        assert!(embedding.is_none());
    }
}