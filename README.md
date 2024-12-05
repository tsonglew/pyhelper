# pyhelper

A collection of Python package utilities written in Rust.

## Features

### Package Conflict Checker

A command-line tool to check if two Python packages have conflicting version requirements.

#### Usage

```bash
# Check if two package versions conflict
cargo run -- --pkg1 "requests>=2.0.0" --pkg2 "requests<3.0.0"

# Examples
cargo run -- --pkg1 "django>=4.0" --pkg2 "django<3.0"  # Conflict
cargo run -- --pkg1 "flask>=2.0" --pkg2 "flask<3.0"    # Compatible
```

#### Supported Version Specifiers

- `>=`: Greater than or equal to
- `<=`: Less than or equal to
- `==`: Exactly equal to
- `<`: Less than
- `>`: Greater than
- `~=`: Compatible release

## Development

### Requirements

- Rust (latest stable version)
- Cargo (comes with Rust)

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## License

MIT
