use once_cell::sync::Lazy;
use std::sync::Mutex;
use fastembed::{TextEmbedding, InitOptions};

static EMBEDDING_MODEL: Lazy<Mutex<TextEmbedding>> = Lazy::new(|| {
    let mut init_options = InitOptions::default();
    init_options.model_name = fastembed::EmbeddingModel::AllMiniLML6V2;
    Mutex::new(TextEmbedding::try_new(init_options).expect("Failed to load embedding model"))
});

pub fn get_embedding(text: &str) -> Vec<f32> {
    let mut model = EMBEDDING_MODEL.lock().unwrap();
    model.embed(vec![text], None).unwrap().remove(0)
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        panic!("Vectors must be the same length");
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimension() {
        let embedding = get_embedding("test");
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }
}