use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bytes::Bytes;

pub struct DB {
    map: Arc<RwLock<HashMap<String, Bytes>>>,
}

impl DB {
    pub fn new() -> Self {
        DB {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: Bytes) {
        let mut map = self.map.write().unwrap();
        map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let map = self.map.read().unwrap();
        map.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_set() {
        let db = Arc::new(DB::new());
        let mut handles = vec![];

        for i in 0..50 {
            let db_clone = Arc::clone(&db);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let value = Bytes::from(format!("value_{}_{}", i, j));
                    db_clone.set(key, value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
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
}