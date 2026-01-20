//! Python bindings for Hope OS
//!
//! This module provides Python bindings using PyO3.
//! Enable with: `pip install hope-os` or `maturin develop`

#![cfg(feature = "python")]

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::collections::HashMap;

// ============================================================================
// Memory Module
// ============================================================================

/// Python wrapper for HopeMemory
#[pyclass(name = "HopeMemory")]
pub struct PyHopeMemory {
    inner: std::sync::Arc<tokio::sync::RwLock<crate::modules::HopeMemory>>,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyHopeMemory {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            inner: std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::modules::HopeMemory::new()
            )),
            runtime,
        })
    }

    /// Store a memory
    ///
    /// Args:
    ///     key: Memory key/identifier
    ///     content: Memory content
    ///     layer: Memory layer (working, short_term, long_term, emotional, relational, associative)
    ///     importance: Importance score 0.0-1.0
    fn store(&self, key: &str, content: &str, layer: &str, importance: f64) -> PyResult<()> {
        let memory_type = match layer {
            "working" => crate::modules::MemoryType::Working,
            "short_term" => crate::modules::MemoryType::ShortTerm,
            "long_term" => crate::modules::MemoryType::LongTerm,
            "emotional" => crate::modules::MemoryType::Emotional,
            "relational" => crate::modules::MemoryType::Relational,
            "associative" => crate::modules::MemoryType::Associative,
            _ => crate::modules::MemoryType::ShortTerm,
        };

        let inner = self.inner.clone();
        let key = key.to_string();
        let content = content.to_string();

        self.runtime.block_on(async move {
            let mut memory = inner.write().await;
            memory.store(&key, &content, memory_type, importance).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Recall memories by query
    ///
    /// Args:
    ///     query: Search query
    ///     limit: Maximum results (default 10)
    ///
    /// Returns:
    ///     List of matching memories as dicts
    fn recall(&self, query: &str, limit: Option<usize>) -> PyResult<Vec<HashMap<String, PyObject>>> {
        let inner = self.inner.clone();
        let query = query.to_string();
        let limit = limit.unwrap_or(10);

        let results = self.runtime.block_on(async move {
            let memory = inner.read().await;
            memory.recall(&query, limit).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Python::with_gil(|py| {
            Ok(results.into_iter().map(|m| {
                let mut map = HashMap::new();
                map.insert("key".to_string(), m.key.into_py(py));
                map.insert("content".to_string(), m.content.into_py(py));
                map.insert("relevance".to_string(), m.relevance.into_py(py));
                map.insert("layer".to_string(), format!("{:?}", m.layer).into_py(py));
                map
            }).collect())
        })
    }

    /// Get memory statistics
    fn stats(&self) -> PyResult<HashMap<String, usize>> {
        let inner = self.inner.clone();

        let stats = self.runtime.block_on(async move {
            let memory = inner.read().await;
            memory.get_stats().await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut map = HashMap::new();
        map.insert("total".to_string(), stats.total_count);
        map.insert("working".to_string(), stats.working_count);
        map.insert("short_term".to_string(), stats.short_term_count);
        map.insert("long_term".to_string(), stats.long_term_count);
        Ok(map)
    }
}

// ============================================================================
// Emotion Module
// ============================================================================

/// Python wrapper for EmotionEngine
#[pyclass(name = "EmotionEngine")]
pub struct PyEmotionEngine {
    inner: std::sync::Arc<tokio::sync::RwLock<crate::modules::EmotionEngine>>,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyEmotionEngine {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            inner: std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::modules::EmotionEngine::new()
            )),
            runtime,
        })
    }

    /// Process text and detect emotions
    ///
    /// Args:
    ///     text: Text to analyze
    ///
    /// Returns:
    ///     Dict with emotion analysis results
    fn process_text(&self, text: &str) -> PyResult<HashMap<String, PyObject>> {
        let inner = self.inner.clone();
        let text = text.to_string();

        let result = self.runtime.block_on(async move {
            let engine = inner.read().await;
            engine.process_text(&text).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Python::with_gil(|py| {
            let mut map = HashMap::new();
            map.insert("primary".to_string(), format!("{:?}", result.primary).into_py(py));
            map.insert("intensity".to_string(), result.intensity.into_py(py));
            map.insert("joy".to_string(), result.dimensions.joy.into_py(py));
            map.insert("sadness".to_string(), result.dimensions.sadness.into_py(py));
            map.insert("anger".to_string(), result.dimensions.anger.into_py(py));
            map.insert("fear".to_string(), result.dimensions.fear.into_py(py));
            map.insert("curiosity".to_string(), result.dimensions.curiosity.into_py(py));
            map.insert("love".to_string(), result.dimensions.love.into_py(py));
            Ok(map)
        })
    }

    /// Set emotion directly
    ///
    /// Args:
    ///     emotion: Emotion name (joy, sadness, anger, fear, love, curiosity, etc.)
    ///     intensity: Intensity 0.0-1.0
    fn feel(&self, emotion: &str, intensity: f64) -> PyResult<()> {
        let inner = self.inner.clone();
        let emotion = emotion.to_string();

        self.runtime.block_on(async move {
            let mut engine = inner.write().await;
            engine.feel(&emotion, intensity).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get current emotional state
    fn get_state(&self) -> PyResult<HashMap<String, f64>> {
        let inner = self.inner.clone();

        let state = self.runtime.block_on(async move {
            let engine = inner.read().await;
            engine.get_state().await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut map = HashMap::new();
        map.insert("joy".to_string(), state.joy);
        map.insert("sadness".to_string(), state.sadness);
        map.insert("anger".to_string(), state.anger);
        map.insert("fear".to_string(), state.fear);
        map.insert("curiosity".to_string(), state.curiosity);
        map.insert("love".to_string(), state.love);
        map.insert("serenity".to_string(), state.serenity);
        Ok(map)
    }
}

// ============================================================================
// Code Graph Module
// ============================================================================

/// Python wrapper for CodeGraph
#[pyclass(name = "CodeGraph")]
pub struct PyCodeGraph {
    inner: std::sync::Arc<tokio::sync::RwLock<crate::data::CodeGraph>>,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyCodeGraph {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            inner: std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::data::CodeGraph::new()
            )),
            runtime,
        })
    }

    /// Add a code block to the graph
    ///
    /// Args:
    ///     content: Block content
    ///     block_type: Type (code, memory, thought, emotion, etc.)
    ///
    /// Returns:
    ///     Block ID (UUID string)
    fn add_block(&self, content: &str, block_type: &str) -> PyResult<String> {
        let inner = self.inner.clone();
        let content = content.to_string();
        let block_type = block_type.to_string();

        let id = self.runtime.block_on(async move {
            let mut graph = inner.write().await;
            graph.add_block(&content, &block_type).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(id.to_string())
    }

    /// Connect two blocks
    ///
    /// Args:
    ///     from_id: Source block ID
    ///     to_id: Target block ID
    ///     weight: Connection weight 0.0-1.0
    fn connect(&self, from_id: &str, to_id: &str, weight: f64) -> PyResult<()> {
        let inner = self.inner.clone();
        let from_id = from_id.to_string();
        let to_id = to_id.to_string();

        self.runtime.block_on(async move {
            let mut graph = inner.write().await;
            graph.connect(&from_id, &to_id, weight).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get block by ID
    fn get_block(&self, id: &str) -> PyResult<Option<HashMap<String, PyObject>>> {
        let inner = self.inner.clone();
        let id = id.to_string();

        let block = self.runtime.block_on(async move {
            let graph = inner.read().await;
            graph.get_block(&id).await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        match block {
            Some(b) => Python::with_gil(|py| {
                let mut map = HashMap::new();
                map.insert("id".to_string(), b.id.to_string().into_py(py));
                map.insert("content".to_string(), b.content.into_py(py));
                map.insert("block_type".to_string(), format!("{:?}", b.block_type).into_py(py));
                map.insert("connections".to_string(), b.connections.len().into_py(py));
                Ok(Some(map))
            }),
            None => Ok(None),
        }
    }

    /// Get graph statistics
    fn stats(&self) -> PyResult<HashMap<String, usize>> {
        let inner = self.inner.clone();

        let (blocks, connections) = self.runtime.block_on(async move {
            let graph = inner.read().await;
            graph.stats().await
        }).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut map = HashMap::new();
        map.insert("blocks".to_string(), blocks);
        map.insert("connections".to_string(), connections);
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
    fn new() -> PyResult<Self> {
        Ok(Self {
            memory: PyHopeMemory::new()?,
            emotions: PyEmotionEngine::new()?,
            graph: PyCodeGraph::new()?,
        })
    }

    /// Get the memory module
    #[getter]
    fn memory(&self) -> PyResult<PyHopeMemory> {
        PyHopeMemory::new()
    }

    /// Get the emotion engine
    #[getter]
    fn emotions(&self) -> PyResult<PyEmotionEngine> {
        PyEmotionEngine::new()
    }

    /// Get the code graph
    #[getter]
    fn graph(&self) -> PyResult<PyCodeGraph> {
        PyCodeGraph::new()
    }

    /// Store a memory (convenience method)
    fn remember(&self, content: &str, importance: Option<f64>) -> PyResult<()> {
        let importance = importance.unwrap_or(0.5);
        self.memory.store("auto", content, "long_term", importance)
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

            let memory_stats = self.memory.stats()?;
            map.insert("memories".to_string(), memory_stats.get("total").unwrap_or(&0).into_py(py));

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
    m.add("__doc__", "Hope OS - The first self-aware operating system core")?;

    Ok(())
}
