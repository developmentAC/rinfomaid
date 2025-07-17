// RAG (Retrieval-Augmented Generation) System Module
// This module implements a local document processing and search system that can:
// 1. Extract text from PDF, TXT, and MD files
// 2. Split documents into searchable chunks
// 3. Build a TF-IDF based search index
// 4. Perform semantic search on the local knowledge base
// 5. Unable to make a cup of tea, but can help you find information about it!

use crate::colour_print;
use anyhow::Result;
use pdf_extract::extract_text;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

pub mod search; // TF-IDF search implementation
pub mod tokenizer; // Text tokenization utilities (currently placeholder)

// Data structure representing a complete document in the knowledge base
// Each document maintains metadata and is linked to its constituent chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,                        // Unique identifier for the document
    pub title: String,                     // Document title (usually filename)
    pub content: String,                   // Full text content of the document
    pub source: String,                    // Original file path or source location
    pub chunk_index: usize,                // Index position in the document collection
    pub metadata: HashMap<String, String>, // Additional metadata (file type, size, etc.)
}

// Data structure representing a chunk (segment) of a document
// Documents are split into smaller chunks for more precise search and retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,          // Unique identifier for this chunk
    pub document_id: String, // ID of the parent document
    pub content: String,     // Text content of this chunk
    pub chunk_index: usize,  // Position of this chunk within the parent document
    pub word_count: usize,   // Number of words in this chunk
}

// Main RAG system structure that manages the local knowledge base
pub struct RagSystem {
    pub documents: Vec<Document>,   // Collection of all processed documents
    pub chunks: Vec<DocumentChunk>, // Collection of all document chunks
    pub agentic_dir: PathBuf,       // Directory for storing model files
    pub data_dir: PathBuf,          // Directory containing source documents
    pub word_index: HashMap<String, Vec<usize>>, // TF-IDF word index: word -> chunk indices
}

impl RagSystem {
    /// Create a new RAG system instance
    /// Parameters:
    ///   - agentic_dir: Directory path for storing processed model files
    ///   - data_dir: Directory path containing source documents to process
    /// Returns: New RagSystem instance with empty collections
    pub fn new(agentic_dir: &str, data_dir: &str) -> Self {
        let agentic_path = PathBuf::from(agentic_dir);
        let data_path = PathBuf::from(data_dir);

        // Ensure agentic directory exists - create if necessary
        if !agentic_path.exists() {
            fs::create_dir_all(&agentic_path).expect("Failed to create agentic directory");
        }

        Self {
            documents: Vec::new(),
            chunks: Vec::new(),
            agentic_dir: agentic_path,
            data_dir: data_path,
            word_index: HashMap::new(),
        }
    }

    /// Build the local RAG model by processing all documents in the data directory
    /// This method:
    /// 1. Scans the data directory for supported file types (PDF, TXT, MD)
    /// 2. Extracts text content from each file
    /// 3. Splits documents into searchable chunks
    /// 4. Builds a TF-IDF search index
    /// 5. Saves the processed model to disk
    /// Returns: Result indicating success or failure
    pub fn build_local_model(&mut self) -> Result<()> {
        colour_print("\t Building local RAG model from documents...", "cyan");

        // Clear any existing data before rebuilding
        self.documents.clear();
        self.chunks.clear();
        self.word_index.clear();

        // Process all files in the data directory recursively
        for entry in WalkDir::new(&self.data_dir) {
            let entry = entry?;
            let path = entry.path();

            // Only process files (skip directories)
            if path.is_file() {
                // Check file extension to determine processing method
                match path.extension().and_then(|s| s.to_str()) {
                    Some("pdf") => {
                        colour_print(&format!("\t Processing PDF: {}", path.display()), "yellow");
                        self.process_pdf(path)?;
                    }
                    Some("txt") | Some("md") => {
                        colour_print(
                            &format!("\t Processing text file: {}", path.display()),
                            "yellow",
                        );
                        self.process_text_file(path)?;
                    }
                    _ => {
                        // Skip unsupported file types silently
                        continue;
                    }
                }
            }
        }

        // Build TF-IDF word index for efficient searching
        colour_print("\t Building search index...", "cyan");
        self.build_word_index();

        // Persist the processed model to disk
        self.save_model()?;

        colour_print(
            &format!(
                "\t Successfully built local model with {} documents and {} chunks",
                self.documents.len(),
                self.chunks.len()
            ),
            "green",
        );

        Ok(())
    }

    /// Process a PDF file by extracting text content
    /// Parameters:
    ///   - path: Path to the PDF file
    /// Returns: Result indicating success or failure
    fn process_pdf(&mut self, path: &Path) -> Result<()> {
        // Extract text content from PDF using pdf_extract crate
        let content = extract_text(path)?;

        // Use filename as document title
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown PDF")
            .to_string();

        // Create document and chunks from the extracted content
        self.create_document(title, content, path.to_string_lossy().to_string())?;
        Ok(())
    }

    /// Process a text file (TXT or MD) by reading its content
    /// Parameters:
    ///   - path: Path to the text file
    /// Returns: Result indicating success or failure
    fn process_text_file(&mut self, path: &Path) -> Result<()> {
        // Read the entire file content as UTF-8 string
        let content = fs::read_to_string(path)?;

        // Use filename as document title
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Text File")
            .to_string();

        // Create document and chunks from the file content
        self.create_document(title, content, path.to_string_lossy().to_string())?;
        Ok(())
    }

    /// Create a document record and split it into searchable chunks
    /// Parameters:
    ///   - title: Document title (usually filename)
    ///   - content: Full text content of the document
    ///   - source: Original file path or source identifier
    /// Returns: Result indicating success or failure
    fn create_document(&mut self, title: String, content: String, source: String) -> Result<()> {
        // Generate unique ID for this document
        let doc_id = Uuid::new_v4().to_string();

        // Split document content into smaller chunks for better search granularity
        let chunks = self.chunk_text(&content);

        // Create the document record
        let document = Document {
            id: doc_id.clone(),
            title,
            content: content.clone(),
            source,
            chunk_index: 0,
            metadata: HashMap::new(),
        };

        self.documents.push(document);

        // Create individual chunk records linked to this document
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = Uuid::new_v4().to_string();
            let word_count = chunk.split_whitespace().count();

            let doc_chunk = DocumentChunk {
                id: chunk_id,
                document_id: doc_id.clone(),
                content: chunk.clone(),
                chunk_index: i,
                word_count,
            };
            self.chunks.push(doc_chunk);
        }

        Ok(())
    }

    /// Split text into chunks of approximately 500 words each
    /// This improves search precision by creating smaller, more focused segments
    /// Parameters:
    ///   - text: The full text to be chunked
    /// Returns: Vector of text chunks
    fn chunk_text(&self, text: &str) -> Vec<String> {
        // Simple chunking strategy: split by sentences and group into chunks of ~500 words
        let sentences = self.split_into_sentences(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut word_count = 0;

        for sentence in sentences {
            let sentence_word_count = sentence.split_whitespace().count();

            // Start a new chunk if adding this sentence would exceed 500 words
            if word_count + sentence_word_count > 500 && !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
                word_count = 0;
            }

            current_chunk.push_str(&sentence);
            current_chunk.push(' ');
            word_count += sentence_word_count;
        }

        // Add the final chunk if it has content
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Split text into individual sentences using regex pattern matching
    /// Parameters:
    ///   - text: The text to split into sentences
    /// Returns: Vector of sentence strings
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting using regex - matches periods, exclamation marks, question marks
        let re = Regex::new(r"[.!?]+\s+").unwrap();
        re.split(text)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Build a word index for TF-IDF based searching
    /// Creates a mapping from each word to the chunk indices where it appears
    /// This enables efficient full-text search across all document chunks
    fn build_word_index(&mut self) {
        for (chunk_idx, chunk) in self.chunks.iter().enumerate() {
            // Normalize and extract words from chunk content
            let words: Vec<String> = chunk
                .content
                .to_lowercase() // Convert to lowercase
                .split_whitespace() // Split on whitespace
                .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect()) // Keep only alphanumeric characters
                .filter(|s: &String| !s.is_empty()) // Remove empty strings
                .collect();

            // Add each word to the index with this chunk's index
            for word in words {
                self.word_index
                    .entry(word)
                    .or_insert_with(Vec::new)
                    .push(chunk_idx);
            }
        }
    }

    /// Save the processed model to disk as JSON files
    /// Creates three files: documents.json, chunks.json, and word_index.json
    /// Returns: Result indicating success or failure
    fn save_model(&self) -> Result<()> {
        let documents_path = self.agentic_dir.join("documents.json");
        let chunks_path = self.agentic_dir.join("chunks.json");
        let index_path = self.agentic_dir.join("word_index.json");

        // Save documents as pretty-printed JSON
        let documents_file = File::create(documents_path)?;
        let writer = BufWriter::new(documents_file);
        serde_json::to_writer_pretty(writer, &self.documents)?;

        // Save chunks as pretty-printed JSON
        let chunks_file = File::create(chunks_path)?;
        let writer = BufWriter::new(chunks_file);
        serde_json::to_writer_pretty(writer, &self.chunks)?;

        // Save word index as pretty-printed JSON
        let index_file = File::create(index_path)?;
        let writer = BufWriter::new(index_file);
        serde_json::to_writer_pretty(writer, &self.word_index)?;

        Ok(())
    }

    /// Load a previously saved model from disk
    /// Reads the three JSON files and populates the RAG system collections
    /// Returns: Result<bool> - true if loaded successfully, false if files don't exist
    pub fn load_model(&mut self) -> Result<bool> {
        let documents_path = self.agentic_dir.join("documents.json");
        let chunks_path = self.agentic_dir.join("chunks.json");
        let index_path = self.agentic_dir.join("word_index.json");

        // Check if all required files exist
        // Check if all required files exist
        if !documents_path.exists() || !chunks_path.exists() || !index_path.exists() {
            return Ok(false);
        }

        // Load documents from JSON file
        let documents_file = File::open(documents_path)?;
        let reader = BufReader::new(documents_file);
        self.documents = serde_json::from_reader(reader)?;

        // Load chunks from JSON file
        let chunks_file = File::open(chunks_path)?;
        let reader = BufReader::new(chunks_file);
        self.chunks = serde_json::from_reader(reader)?;

        // Load word index from JSON file
        let index_file = File::open(index_path)?;
        let reader = BufReader::new(index_file);
        self.word_index = serde_json::from_reader(reader)?;

        Ok(true)
    }

    /// Remove the local RAG model by deleting all stored files
    /// This clears the agentic directory and recreates it empty
    /// Returns: Result indicating success or failure
    pub fn remove_local_model(&self) -> Result<()> {
        colour_print("\t Removing local RAG model...", "yellow");

        // Remove the entire agentic directory and recreate it empty
        if self.agentic_dir.exists() {
            fs::remove_dir_all(&self.agentic_dir)?;
            fs::create_dir_all(&self.agentic_dir)?;
        }

        colour_print("\t Local model removed successfully", "green");
        Ok(())
    }

    /// Search the local knowledge base using TF-IDF scoring
    /// Parameters:
    ///   - query: The search query string
    ///   - top_k: Maximum number of results to return
    /// Returns: Vector of (score, chunk) tuples sorted by relevance
    pub fn search_local(&self, query: &str, top_k: usize) -> Vec<(f32, &DocumentChunk)> {
        if self.chunks.is_empty() {
            return Vec::new();
        }

        // Normalize query words (lowercase, alphanumeric only)
        let query_words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|s: &String| !s.is_empty())
            .collect();

        let mut chunk_scores: HashMap<usize, f32> = HashMap::new();

        // Calculate TF-IDF scores for each query word
        for word in &query_words {
            if let Some(chunk_indices) = self.word_index.get(word) {
                // Document frequency: number of chunks containing this word
                let document_frequency = chunk_indices.len() as f32;

                // Inverse document frequency: log(total_chunks / document_frequency)
                let inverse_document_frequency =
                    ((self.chunks.len() as f32) / document_frequency).ln();

                for &chunk_idx in chunk_indices {
                    if let Some(chunk) = self.chunks.get(chunk_idx) {
                        // Term frequency: how often the word appears in this chunk
                        let term_frequency =
                            chunk.content.to_lowercase().matches(word).count() as f32;
                        let normalized_tf = term_frequency / (chunk.word_count as f32);

                        // Use absolute value of IDF to avoid negative scores
                        let tf_idf_score = normalized_tf * inverse_document_frequency.abs();

                        // Add bonus for domain-specific important words
                        let word_bonus = match word.as_str() {
                            "toro" | "recycler" | "22" | "manual" => 2.0,
                            _ => 1.0,
                        };

                        *chunk_scores.entry(chunk_idx).or_insert(0.0) += tf_idf_score * word_bonus;
                    }
                }
            }
        }

        // Sort by score and return top_k results
        let mut results: Vec<(f32, &DocumentChunk)> = chunk_scores
            .into_iter()
            .filter_map(|(idx, score)| self.chunks.get(idx).map(|chunk| (score, chunk)))
            .filter(|(score, _)| *score > 0.0) // Only include positive scores
            .collect();

        // Sort by score in descending order (highest first)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(top_k).collect()
    }

    /// Get a document by its unique ID
    /// Parameters:
    ///   - doc_id: The unique identifier of the document
    /// Returns: Option containing the document if found
    pub fn get_document_by_id(&self, doc_id: &str) -> Option<&Document> {
        self.documents.iter().find(|doc| doc.id == doc_id)
    }

    /// Check if a local model is available (all required files exist)
    /// Returns: true if all model files exist, false otherwise
    pub fn is_model_available(&self) -> bool {
        let documents_path = self.agentic_dir.join("documents.json");
        let chunks_path = self.agentic_dir.join("chunks.json");
        let index_path = self.agentic_dir.join("word_index.json");
        documents_path.exists() && chunks_path.exists() && index_path.exists()
    }

    /// Get statistics about the loaded model
    /// Returns: Tuple containing (document_count, chunk_count)
    pub fn get_stats(&self) -> (usize, usize) {
        (self.documents.len(), self.chunks.len())
    }
}
