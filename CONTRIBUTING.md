# Contributing to miPC

Thank you for your interest in contributing to miPC! This document outlines the process for contributing to the project.

## Development Setup

### Prerequisites

- **Node.js** 20+ and npm
- **Rust** stable toolchain (install via [rustup](https://rustup.rs/))
- **Windows 10/11** (this is a Windows-only application)
- **Visual Studio Build Tools** with C++ workload
- **Windows SDK** (for Windows API access)

### Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/mafsc/micontrol.git
   cd micontrol
   ```

2. Install frontend dependencies:

   ```bash
   npm install
   ```

3. Run the development server:
   ```bash
   npm run tauri dev
   ```

### Project Structure

```
micontrol/
├── src/                    # React frontend (TypeScript)
│   ├── components/          # Reusable UI components
│   ├── hooks/              # Custom React hooks
│   ├── pages/              # Page/tab components
│   ├── i18n/               # Internationalization
│   └── styles/             # CSS and styling
├── src-tauri/              # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── hw/             # Hardware abstraction layer (HAL)
│   │   ├── commands/       # Tauri command handlers
│   │   ├── util/           # Utility modules
│   │   └── lib.rs          # Application entry point
│   └── Cargo.toml          # Rust dependencies
├── .github/workflows/      # CI/CD pipelines
└── docs/                   # Documentation
```

### Development Workflow

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes
3. Run the health check:
   ```bash
   cargo fmt --manifest-path src-tauri/Cargo.toml --check
   cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
   cargo test --manifest-path src-tauri/Cargo.toml
   npx tsc --noEmit
   npm run lint
   npm run build
   ```
4. Commit with a conventional commit message
5. Push and create a pull request

### Commit Message Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Build/tooling changes

### Code Style

#### Rust

- Run `cargo fmt` before committing
- Zero clippy warnings allowed (`-D warnings`)
- Add tests for new hardware modules
- Use `HardwareResult<T>` for hardware functions

#### TypeScript/React

- Run `npm run lint` before committing
- Use functional components with hooks
- Add TypeScript types for all props
- Use the i18n system for all user-facing strings

### Testing

- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Frontend tests: `npm test`
- Integration tests: `cargo test --manifest-path src-tauri/Cargo.toml --test '*'`

### Pull Request Process

1. Ensure all health checks pass
2. Update documentation if needed
3. Add a changelog entry
4. Request review from maintainers

## Questions?

Feel free to open an issue for any questions about contributing.
