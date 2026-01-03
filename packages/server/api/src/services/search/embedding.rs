use anyhow::Result;

// ARCHITECTURE NOTE:
// We are currently using a MOCK Embedder to bypass 'ort' compilation issues on MacOS.
// The Interface is "Future Proof". When ready, we simply uncomment 'fastembed'
// and swap this struct with the real one. The rest of the app (database, engine)
// won't know the difference.

pub struct LocalEmbedder;

impl LocalEmbedder {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Generate embeddings for a single string.
    /// Returns vector(384) - Matching 'all-MiniLM-L6-v2' dimension.
    pub fn generate(&self, _text: &str) -> Result<Vec<f32>> {
        // Return a zero vector of dimension 384.
        // This is valid for pgvector, but will result in 0 cosine similarity.
        Ok(vec![0.0; 384])
    }
}
