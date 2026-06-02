# Contributing to RGUI

Thank you for your interest in contributing to RGUI!

RGUI is an open-source native GUI framework for Rust powered by wgpu. The project is still evolving rapidly, and contributions of all kinds are welcome.

## Ways to Contribute

You can contribute by:

* Reporting bugs
* Improving documentation
* Suggesting new features
* Implementing features
* Writing examples and tutorials
* Improving performance
* Reviewing pull requests
* Testing on different platforms

## Before You Start

Please check existing issues and pull requests before opening a new one.

For larger changes, we recommend opening an issue first to discuss the proposed design and implementation approach.

## Development Setup

Clone the repository:

```bash
git clone https://github.com/<your-org>/rgui.git
cd rgui
```

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Format the code:

```bash
cargo fmt
```

Run lints:

```bash
cargo clippy --all-targets --all-features
```

## Coding Guidelines

### Rust Style

* Follow standard Rust formatting (`cargo fmt`).
* Ensure code passes Clippy without warnings whenever possible.
* Prefer explicit and readable code over clever abstractions.
* Avoid unnecessary allocations and clones.
* Keep APIs strongly typed whenever practical.

### Architecture

RGUI prioritizes:

* Native-first design
* Strong typing
* Performance
* Predictable APIs
* Extensibility
* Minimal runtime overhead

When proposing new features, consider how they affect these goals.

## Pull Requests

Before submitting a pull request:

* Ensure the project builds successfully.
* Run tests.
* Run formatting and linting tools.
* Add documentation when introducing public APIs.
* Include examples when appropriate.

Pull requests should focus on a single logical change whenever possible.

## Issues

When reporting a bug, please include:

* Operating system
* Rust version
* Steps to reproduce
* Expected behavior
* Actual behavior
* Relevant logs or screenshots

## Documentation Contributions

Documentation improvements are highly valued.

Examples, tutorials, architecture explanations, and API documentation are all useful contributions.

## Design Discussions

RGUI is still in active development and some subsystems may change significantly.

Constructive discussions about architecture, APIs, rendering, layout, widgets, and developer experience are encouraged.

## Code of Conduct

Be respectful and professional.

We welcome contributors from all backgrounds and experience levels. Harassment, discrimination, or hostile behavior will not be tolerated.

## License

By contributing to RGUI, you agree that your contributions will be licensed under the same license(s) as the project.

---

