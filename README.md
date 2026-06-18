# copper

A small command-line unit converter written in Rust.

Copper converts a quantity from one unit to another.
Units don't need a direct conversion defined between them; we search the graph of known conversions (breadth-first) to find a suitable conversion path.

## Usage

```
copper <QUANTITY> <FROM> <TO>
```

## Supported units

* Length: `mm`, `cm`, `m`, `km`, `in`, `ft`, `yd`, `mi`
* Temperature: `C` (Celsius), `K` (Kelvin), `F` (Fahrenheit), `R` (Rankine)
* Data:
  * Base: `b` (bits), `B` (bytes)
  * Metric (powers of 1000): `kb`/`kB`, `Mb`/`MB`, `Gb`/`GB`, `Tb`/`TB`
  * IEC (powers of 1024): `Kib`/`KiB`, `Mib`/`MiB`, `Gib`/`GiB`, `Tib`/`TiB`

## Features/roadmap

* [ ] Broad unit support
* [x] Path finding between units without explicit conversions
* [x] Natural CLI syntax
* [ ] Output customization
* [ ] Equation solver to avoid dropping to float until absolutely necessary

## Building

```sh
cargo build --release
```

Or just

```sh
cargo run -- 3 m ft
```

## Project structure

* `src/main.rs`: the CLI (argument parsing via clap), unit definitions, the conversion table, and the path-finding logic).
* `libcopper/`: a procedural macro crate providing `make_table!`, a DSL for declaring conversions. Each row reads like
  `Unit::Metre -> { Unit::Kilometre => div KILO }`,
  and the macro generates the lookup table plus the inverse of every arithmetic conversion (`mul`/`div` and `add`/`sub` pairs; `fun(...)` closures have no automatic inverse).

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
