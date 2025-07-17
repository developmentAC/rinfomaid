use anyhow::Result;
use std::collections::HashMap;

pub struct EmbeddingModel {
    vocab_size: usize,
    embedding_dim: usize,
}

impl EmbeddingModel {
    pub fn new(vocab_size: usize, embedding_dim: usize) -> Self {
        Self {
            vocab_size,
            embedding_dim,
        }
    }

    pub fn encode_text(&self, text: &str, vocab: &HashMap<String, usize>) -> Result<Vec<f32>> {
        let mut embedding = vec![0.0; self.embedding_dim];
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut word_count = HashMap::new();
        for word in &words {
            *word_count.entry(word.clone()).or_insert(0) += 1;
        }

        // Simple TF-IDF like encoding
        for (word, count) in word_count {
            if let Some(&word_idx) = vocab.get(&word) {
                if word_idx < self.embedding_dim {
                    embedding[word_idx] = count as f32 / words.len() as f32;
                }
            }
        }

        Ok(embedding)
    }

    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }
}
