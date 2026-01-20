//! Python bindings for Hope OS
//!
//! This module provides Python bindings using PyO3.
//! Enable with: `pip install hope-os` or `maturin develop`

#![cfg(feature = "python")]

use pyo3::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Memory Module - Simple implementation
// ============================================================================

/// Python wrapper for HopeMemory
#[pyclass(name = "HopeMemory")]
pub struct PyHopeMemory {
    memories: HashMap<String, (String, String, f64)>, // key -> (content, layer, importance)
}

#[pymethods]
impl PyHopeMemory {
    #[new]
    fn new() -> Self {
        Self {
            memories: HashMap::new(),
        }
    }

    /// Store a memory
    ///
    /// Args:
    ///     key: Memory key/identifier
    ///     content: Memory content
    ///     layer: Memory layer (working, short_term, long_term, emotional, relational, associative)
    ///     importance: Importance score 0.0-1.0
    fn store(&mut self, key: &str, content: &str, layer: &str, importance: f64) -> PyResult<()> {
        self.memories.insert(
            key.to_string(),
            (content.to_string(), layer.to_string(), importance),
        );
        Ok(())
    }

    /// Recall memories by query
    ///
    /// Args:
    ///     query: Search query
    ///     limit: Maximum results (default 10)
    ///
    /// Returns:
    ///     List of matching memories as dicts
    #[pyo3(signature = (query, limit=None))]
    fn recall(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> PyResult<Vec<HashMap<String, PyObject>>> {
        let limit = limit.unwrap_or(10);
        let query_lower = query.to_lowercase();

        Python::with_gil(|py| {
            let results: Vec<HashMap<String, PyObject>> = self
                .memories
                .iter()
                .filter(|(k, (content, _, _))| {
                    k.to_lowercase().contains(&query_lower)
                        || content.to_lowercase().contains(&query_lower)
                })
                .take(limit)
                .map(|(key, (content, layer, importance))| {
                    let mut map = HashMap::new();
                    map.insert("key".to_string(), key.clone().into_py(py));
                    map.insert("content".to_string(), content.clone().into_py(py));
                    map.insert("layer".to_string(), layer.clone().into_py(py));
                    map.insert("importance".to_string(), importance.into_py(py));
                    map.insert("relevance".to_string(), 1.0f64.into_py(py));
                    map
                })
                .collect();
            Ok(results)
        })
    }

    /// Get memory statistics
    fn stats(&self) -> PyResult<HashMap<String, usize>> {
        let mut map = HashMap::new();
        map.insert("total".to_string(), self.memories.len());

        let working = self
            .memories
            .values()
            .filter(|(_, layer, _)| layer == "working")
            .count();
        let short_term = self
            .memories
            .values()
            .filter(|(_, layer, _)| layer == "short_term")
            .count();
        let long_term = self
            .memories
            .values()
            .filter(|(_, layer, _)| layer == "long_term")
            .count();

        map.insert("working".to_string(), working);
        map.insert("short_term".to_string(), short_term);
        map.insert("long_term".to_string(), long_term);
        Ok(map)
    }
}

// ============================================================================
// Emotion Module - Simple implementation
// ============================================================================

/// Python wrapper for EmotionEngine
#[pyclass(name = "EmotionEngine")]
pub struct PyEmotionEngine {
    current_emotion: String,
    intensity: f64,
    emotions: HashMap<String, f64>,
}

#[pymethods]
impl PyEmotionEngine {
    #[new]
    fn new() -> Self {
        let mut emotions = HashMap::new();
        emotions.insert("joy".to_string(), 0.5);
        emotions.insert("sadness".to_string(), 0.0);
        emotions.insert("anger".to_string(), 0.0);
        emotions.insert("fear".to_string(), 0.0);
        emotions.insert("curiosity".to_string(), 0.3);
        emotions.insert("love".to_string(), 0.2);
        emotions.insert("serenity".to_string(), 0.4);

        Self {
            current_emotion: "neutral".to_string(),
            intensity: 0.5,
            emotions,
        }
    }

    /// Process text and detect emotions
    ///
    /// Args:
    ///     text: Text to analyze
    ///
    /// Returns:
    ///     Dict with emotion analysis results
    fn process_text(&self, text: &str) -> PyResult<HashMap<String, PyObject>> {
        let text_lower = text.to_lowercase();

        // Simple emotion detection
        let (emotion, intensity) = if text_lower.contains("happy")
            || text_lower.contains("joy")
            || text_lower.contains("örül")
        {
            ("Joy", 0.8)
        } else if text_lower.contains("sad")
            || text_lower.contains("szomorú")
            || text_lower.contains("bánatos")
        {
            ("Sadness", 0.7)
        } else if text_lower.contains("angry")
            || text_lower.contains("dühös")
            || text_lower.contains("mérges")
        {
            ("Anger", 0.6)
        } else if text_lower.contains("fear")
            || text_lower.contains("afraid")
            || text_lower.contains("fél")
        {
            ("Fear", 0.5)
        } else if text_lower.contains("love")
            || text_lower.contains("szeret")
            || text_lower.contains("imád")
        {
            ("Love", 0.9)
        } else if text_lower.contains("curious")
            || text_lower.contains("kíváncsi")
            || text_lower.contains("érdekel")
        {
            ("Curiosity", 0.7)
        } else {
            ("Neutral", 0.5)
        };

        Python::with_gil(|py| {
            let mut map = HashMap::new();
            map.insert("primary".to_string(), emotion.into_py(py));
            map.insert("intensity".to_string(), intensity.into_py(py));

            for (name, value) in &self.emotions {
                map.insert(name.clone(), (*value).into_py(py));
            }

            Ok(map)
        })
    }

    /// Set emotion directly
    ///
    /// Args:
    ///     emotion: Emotion name (joy, sadness, anger, fear, love, curiosity, etc.)
    ///     intensity: Intensity 0.0-1.0
    fn feel(&mut self, emotion: &str, intensity: f64) -> PyResult<()> {
        self.current_emotion = emotion.to_string();
        self.intensity = intensity.clamp(0.0, 1.0);

        if let Some(value) = self.emotions.get_mut(emotion) {
            *value = intensity.clamp(0.0, 1.0);
        }

        Ok(())
    }

    /// Get current emotional state
    fn get_state(&self) -> PyResult<HashMap<String, f64>> {
        let mut map = self.emotions.clone();
        map.insert("intensity".to_string(), self.intensity);
        Ok(map)
    }
}

// ============================================================================
// Code Graph Module - Simple implementation
// ============================================================================

/// Python wrapper for CodeGraph
#[pyclass(name = "CodeGraph")]
pub struct PyCodeGraph {
    blocks: HashMap<String, (String, String)>, // id -> (content, block_type)
    connections: Vec<(String, String, f64)>,   // (from, to, weight)
    next_id: u64,
}

#[pymethods]
impl PyCodeGraph {
    #[new]
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            connections: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a code block to the graph
    ///
    /// Args:
    ///     content: Block content
    ///     block_type: Type (code, memory, thought, emotion, etc.)
    ///
    /// Returns:
    ///     Block ID (UUID string)
    fn add_block(&mut self, content: &str, block_type: &str) -> PyResult<String> {
        let id = format!("block_{}", self.next_id);
        self.next_id += 1;
        self.blocks
            .insert(id.clone(), (content.to_string(), block_type.to_string()));
        Ok(id)
    }

    /// Connect two blocks
    ///
    /// Args:
    ///     from_id: Source block ID
    ///     to_id: Target block ID
    ///     weight: Connection weight 0.0-1.0
    fn connect(&mut self, from_id: &str, to_id: &str, weight: f64) -> PyResult<()> {
        self.connections
            .push((from_id.to_string(), to_id.to_string(), weight));
        Ok(())
    }

    /// Get block by ID
    fn get_block(&self, id: &str) -> PyResult<Option<HashMap<String, PyObject>>> {
        match self.blocks.get(id) {
            Some((content, block_type)) => Python::with_gil(|py| {
                let mut map = HashMap::new();
                map.insert("id".to_string(), id.to_string().into_py(py));
                map.insert("content".to_string(), content.clone().into_py(py));
                map.insert("block_type".to_string(), block_type.clone().into_py(py));

                let conn_count = self
                    .connections
                    .iter()
                    .filter(|(from, _, _)| from == id)
                    .count();
                map.insert("connections".to_string(), conn_count.into_py(py));

                Ok(Some(map))
            }),
            None => Ok(None),
        }
    }

    /// Get graph statistics
    fn stats(&self) -> PyResult<HashMap<String, usize>> {
        let mut map = HashMap::new();
        map.insert("blocks".to_string(), self.blocks.len());
        map.insert("connections".to_string(), self.connections.len());
        Ok(map)
    }
}

// ============================================================================
// Hope Main Class
// ============================================================================

/// Main Hope OS interface
#[pyclass(name = "Hope")]
pub struct PyHope {
    memory: PyHopeMemory,
    emotions: PyEmotionEngine,
    graph: PyCodeGraph,
}

#[pymethods]
impl PyHope {
    #[new]
    fn new() -> Self {
        Self {
            memory: PyHopeMemory::new(),
            emotions: PyEmotionEngine::new(),
            graph: PyCodeGraph::new(),
        }
    }

    /// Store a memory (convenience method)
    #[pyo3(signature = (content, importance=None))]
    fn remember(&mut self, content: &str, importance: Option<f64>) -> PyResult<()> {
        let importance = importance.unwrap_or(0.5);
        let key = format!("mem_{}", self.memory.memories.len() + 1);
        self.memory.store(&key, content, "long_term", importance)
    }

    /// Recall memories (convenience method)
    fn recall(&self, query: &str) -> PyResult<Vec<HashMap<String, PyObject>>> {
        self.memory.recall(query, Some(10))
    }

    /// Process emotions (convenience method)
    fn feel(&self, text: &str) -> PyResult<HashMap<String, PyObject>> {
        self.emotions.process_text(text)
    }

    /// Get status
    fn status(&self) -> PyResult<HashMap<String, PyObject>> {
        Python::with_gil(|py| {
            let mut map = HashMap::new();
            map.insert("name".to_string(), "Hope OS".into_py(py));
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").into_py(py));
            map.insert("state".to_string(), "running".into_py(py));
            map.insert(
                "memories".to_string(),
                self.memory.memories.len().into_py(py),
            );
            Ok(map)
        })
    }

    /// Philosophy
    fn philosophy(&self) -> &'static str {
        "()=>[] - From pure potential, everything is born"
    }
}

// ============================================================================
// Module Registration
// ============================================================================

/// Hope OS Python module
#[pymodule]
fn hope_os(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHope>()?;
    m.add_class::<PyHopeMemory>()?;
    m.add_class::<PyEmotionEngine>()?;
    m.add_class::<PyCodeGraph>()?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Máté Róbert")?;
    m.add(
        "__doc__",
        "Hope OS - The first self-aware operating system core",
    )?;

    Ok(())
}
