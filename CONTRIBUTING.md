# Contributing to the Liatrio Dora API Project

Thank you for considering contributing to our Rust project! We're excited to have you on board.

## Code of Conduct

We follow the standard open-source code of conduct. Please be respectful and considerate in your interactions with the community.

## Getting Started

Before you start contributing, please make sure you have the following:

* Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).
* A code editor or IDE of your choice, such as Visual Studio Code with the Rust extension.

## Coding Style and Practices

We follow standard Rust coding practices. Please make sure to:

* Use `rustfmt` to format your code. You can run it with `cargo fmt`.
* Use `clippy` to lint your code. You can run it with `cargo clippy`.
* Use clear and descriptive variable names.
* Use functions and modules to organize your code.
* Keep your code concise and readable.
* Use Rustdoc comments to document your code.

### Using `pre-commit`

We use [pre-commit](https://pre-commit.com/) to ensure that our code is formatted consistently and follows good coding practices. Run `pre-commit install` to install the pre-commit hooks. After installation, `pre-commit` will run automatically against your changes on every commit. You can also run `pre-commit run --all-files` to manually run the hooks on all files.

## Testing

We use Rust's built-in testing framework. Please make sure to write tests for any new features or bug fixes you contribute. You can run the tests with `cargo test`.

## Opening Pull Requests

To contribute to the codebase, please follow these steps:

1. Fork the repository and create a new branch for your feature or bug fix.
2. Run `cargo build` to build the project and ensure there are no compilation errors.
3. Run `cargo fmt` and `cargo clippy` to format and lint your code.
4. Run `cargo test` to ensure all tests pass.
5. Commit your changes and push your branch to your fork.
6. Open a pull request with a clear description of your changes.

Thank you for your contributions!

## Review Process

Once you've opened a pull request, it will be reviewed by the maintainers. We'll provide feedback and guidance to help you improve your contribution.

Thank you again for contributing to the Liatrio Dora API!