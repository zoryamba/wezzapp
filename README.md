## wezzapp – A Cross-Platform Weather CLI (Rust)

wezzapp is a cross-platform command-line weather application written in Rust.
It supports multiple weather providers (WeatherAPI, AccuWeather), interactive credential configuration, and a pluggable
architecture ready for expansion.

The project is structured as a Cargo workspace with separate crates for:

wezzapp-core — domain logic, provider abstractions, configuration storage

wezzapp-cli — the command-line interface, argument parsing, and user interactions

The goal of this project is to demonstrate clean architecture, testability, error-handling discipline, and idiomatic
Rust design practices.

## Project Structure

```
wezzapp/
├── Cargo.toml            # Workspace root
├── Cargo.lock            # Workspace root
├── crates/
│   ├── wezzapp-core/     # Core logic: providers, store, clients
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   └── wezzapp-cli/      # CLI binary
│       ├── src/
│       │   ├── main.rs
│       │   └── ...
│       └── Cargo.toml
└── README.md
```

## Installation

```bash
$ git clone <repo>
$ cd wezzapp
$ cargo build --release
```

CLI will be available at `target/release/wezzapp`

## Usage

### 1. Configure credentials

```bash
$ wezzapp configure weatherapi
# OR
$ wezzapp configure accuweather
```

You will be prompted interactively:

- API key
- Whether to overwrite existing credentials
- Whether to set the provider as default

### 2. Fetch weather forecast

```bash
# fetch weather for today with the default provider
$ wezzapp get "Kyiv, Ukraine"

# fetch weather for specific date
$ wezzapp get "Kyiv, Ukraine" "2021-05-20"

# fetch weather for specific provider
$ wezzapp get "Kyiv, Ukraine" --provider accuweather
```

## Config file location

Credentials are stored in:

Linux/macOS:
$HOME/.wezzapp/credentials.toml

Windows:
{FOLDERID_Profile}\.wezzapp\credentials.toml

Tested on macOS only, don't have Windows machine.

```toml
default = "weatherapi"

[providers.weatherapi.weatherapi]
api_key = "******"

[providers.accuweather.accuweather]
api_key = "******"
```

Provider keys may seem repetitive, but this structure may be useful if in future we decide to store multiple credentials
for the same customer under some custom alias (I would definitely do, if I had free time :))

## Testing

To run tests:

```bash
$ cargo test --all
```

## Development

Run clippy:

```bash
$ cargo clippy --all-targets
```

Run formatter:

```bash
$ cargo fmt --all
```

Generate docs:

```bash 
$ cargo doc --workspace --open
```

Run application in debug mode:

```bash
RUST_LOG=debug cargo run -- get "Kyiv, Ukraine"
```

## Future Improvements

 - Cover API clients with tests
 - Use `secrecy` crate for handling credentials
 - Configure handler uses store directly, maybe introduce some credentials service in core crate
 - Consider some crate for dependency injection