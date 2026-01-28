use once_cell::sync::Lazy;
use fastembed::{TextEmbedding, InitOptions};

static EMBEDDING_MODEL: Lazy<TextEmbedding> = Lazy::new(|| {
    TextEmbedding::try_new(InitOptions {
        model_name: fastembed::EmbeddingModel::AllMiniLML6V2,
        ..Default::default()
    }).expect("Failed to load embedding model")
});

pub fn get_embedding(text: &str) -> Vec<f32> {
    EMBEDDING_MODEL.embed(vec![text], None).unwrap().remove(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimension() {
        let embedding = get_embedding("test");
        assert_eq!(embedding.len(), 384);
    }
}