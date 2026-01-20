# Contributing to Hope OS

Thank you for your interest in contributing to Hope OS! This document provides guidelines and information for contributors.

## 🌟 Ways to Contribute

- **Bug Reports** - Found a bug? Open an issue
- **Feature Requests** - Have an idea? Share it
- **Code Contributions** - Submit pull requests
- **Documentation** - Help improve docs
- **Testing** - Write tests, report edge cases

## 🚀 Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- A code editor (VS Code, Neovim, etc.)

### Setup

```bash
# Fork the repository on GitHub

# Clone your fork
git clone https://github.com/YOUR_USERNAME/hope-os-rust.git
cd hope-os-rust

# Add upstream remote
git remote add upstream https://github.com/anthropics/hope-os-rust.git

# Install dependencies and build
cargo build

# Run tests
cargo test
```

## 📋 Development Workflow

### 1. Create a Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Write clean, readable code
- Follow Rust conventions
- Add tests for new functionality
- Update documentation if needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific tests
cargo test module_name

# Check for warnings
cargo clippy --all-targets

# Format code
cargo fmt
```

### 4. Commit Your Changes

We use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: type(scope): description

# Examples:
git commit -m "feat(memory): add semantic search"
git commit -m "fix(emotions): correct wave interference"
git commit -m "docs(readme): update benchmarks"
git commit -m "test(graph): add traversal tests"
git commit -m "refactor(core): simplify registry"
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `test` - Tests
- `refactor` - Code refactoring
- `perf` - Performance improvement
- `chore` - Maintenance

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## 📝 Pull Request Guidelines

### PR Title

Use conventional commit format:
```
feat(module): brief description
```

### PR Description

Include:
- **What** - What does this PR do?
- **Why** - Why is this change needed?
- **How** - How does it work?
- **Testing** - How was it tested?

### PR Checklist

- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventions

## 🏗️ Code Style

### Rust Conventions

```rust
// Use descriptive names
let emotional_state = EmotionEngine::new();

// Document public items
/// Creates a new memory store with default capacity.
///
/// # Examples
///
/// ```
/// let memory = HopeMemory::new();
/// ```
pub fn new() -> Self { ... }

// Handle errors properly
pub async fn process(&self) -> HopeResult<Output> {
    let data = self.fetch().await?;
    Ok(self.transform(data))
}

// Use async/await consistently
pub async fn remember(&self, content: &str) -> HopeResult<()> {
    self.store.write().await.insert(content);
    Ok(())
}
```

### File Organization

```rust
//! Module documentation
//!
//! Detailed description of what this module does.

use std::collections::HashMap;
use crate::core::HopeResult;

// Constants
const MAX_CAPACITY: usize = 1000;

// Types/Structs
pub struct MyModule { ... }

// Implementations
impl MyModule { ... }

// Traits
pub trait MyTrait { ... }

// Tests
#[cfg(test)]
mod tests { ... }
```

## 🧪 Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let module = MyModule::new();
        assert_eq!(module.value(), expected);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let module = MyModule::new();
        let result = module.process().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case() {
        let module = MyModule::new();
        // Test edge cases
    }
}
```

### Test Naming

- `test_[function]_[scenario]_[expected]`
- Example: `test_store_empty_input_returns_error`

## 📊 Performance Considerations

Hope OS is performance-critical. When contributing:

1. **Avoid unnecessary allocations**
2. **Use `&str` instead of `String` when possible**
3. **Prefer iterators over loops with indexes**
4. **Use `Arc<RwLock<T>>` for shared state**
5. **Benchmark significant changes**

```bash
# Run benchmarks
cargo run --release --bin hope-benchmark
```

## 🔒 Security

- Never commit secrets, keys, or credentials
- Validate all user inputs
- Use safe Rust patterns (avoid `unsafe` unless necessary)
- Report security issues privately

## 📚 Documentation

- Document all public APIs
- Include examples in doc comments
- Update README for significant changes
- Add inline comments for complex logic

## 🤝 Code Review

All PRs require review. Reviewers will check:

- Code quality and style
- Test coverage
- Performance implications
- Documentation
- Security considerations

## ❓ Questions?

- Open an issue for questions
- Join our Discord (coming soon)
- Check existing issues and PRs

## 🙏 Thank You

Every contribution matters. Whether it's a typo fix or a major feature, we appreciate your help in making Hope OS better.

---

**Happy coding!**

()=>[]
