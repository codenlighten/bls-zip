# Contributing to Boundless BLS Platform

Thank you for your interest in contributing to the Boundless BLS Platform! This document provides guidelines and instructions for contributing.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/boundless-bls-platform.git
   cd boundless-bls-platform
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/Saifullah62/BLS.git
   ```
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites
- **Rust**: 1.75 or later
- **Node.js**: 20.x or later
- **PostgreSQL**: 14 or later (for E² Multipass)
- **Git**: Latest stable version

### Blockchain & RPC Setup
```bash
# Build the blockchain node
cargo build --release --bin boundless-node

# Run tests
cargo test
```

### E² Multipass Setup
```bash
# Navigate to enterprise directory
cd enterprise

# Build backend
cargo build --release --bin enterprise-server

# Setup frontend
cd frontend
npm install
npm run dev
```

### BLS Explorer Setup
```bash
# Navigate to explorer directory
cd BLS_Explorer

# Install dependencies
npm install

# Run development server
npm run dev
```

## Making Changes

### Branch Naming
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring
- `test/description` - Test additions/updates
- `chore/description` - Maintenance tasks

### Commit Messages
Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): subject

body (optional)

footer (optional)
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Test additions/updates
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes

**Examples:**
```
feat(rpc): add chain_getTransaction endpoint

Implements RPC endpoint for retrieving transaction details by hash.
Includes input/output details and block information.

Closes #123
```

```
fix(core): resolve signature verification edge case

Fixed issue where hybrid signatures failed validation
in specific multi-sig scenarios.
```

## Submitting Changes

### Before Submitting
1. **Update your branch** with latest upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all tests**:
   ```bash
   # Rust tests
   cargo test --all

   # Node.js tests (if applicable)
   cd BLS_Explorer && npm test
   cd enterprise/frontend && npm test
   ```

3. **Run linters**:
   ```bash
   # Rust
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings

   # TypeScript
   npm run lint
   ```

4. **Update documentation** if needed

### Creating a Pull Request
1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a Pull Request** on GitHub

3. **Fill out the PR template** completely

4. **Wait for review** - Maintainers will review your PR and may request changes

### PR Review Process
- **All tests must pass** (CI/CD checks)
- **At least one approval** from maintainers required
- **Code must follow** style guidelines
- **Documentation must be updated** if applicable
- **No merge conflicts** with main branch

## Code Style

### Rust
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Maximum line length: 100 characters
- Use meaningful variable and function names
- Add comments for complex logic

### TypeScript/JavaScript
- Follow project's ESLint configuration
- Use TypeScript for type safety
- Prefer functional components (React)
- Use meaningful variable and function names
- Add JSDoc comments for public APIs

### Documentation
- Update README files when adding features
- Add inline code comments for complex logic
- Update API documentation when changing interfaces
- Include examples where helpful

## Testing

### Rust Tests
```bash
# Run all tests
cargo test

# Run specific package tests
cargo test --package boundless-core

# Run with output
cargo test -- --nocapture
```

### Integration Tests
- Write integration tests for new features
- Test error cases and edge conditions
- Ensure tests are deterministic

### Manual Testing
- Test on multiple platforms (Linux, Windows, macOS) when possible
- Verify backward compatibility
- Test performance implications

## Documentation

### Code Documentation
- Add rustdoc comments to public APIs
- Include examples in documentation
- Document error conditions
- Explain complex algorithms

### User Documentation
- Update README files
- Add usage examples
- Document configuration options
- Update CHANGELOG

## Security

If you discover a security vulnerability:
1. **Do NOT** open a public issue
2. Email security@boundlesstrust.org
3. See [SECURITY.md](SECURITY.md) for details

## Questions?

- Open a [Discussion](https://github.com/Saifullah62/BLS/discussions)
- Check existing [Issues](https://github.com/Saifullah62/BLS/issues)
- See [SUPPORT.md](SUPPORT.md) for support channels

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

Thank you for contributing to Boundless BLS Platform!
