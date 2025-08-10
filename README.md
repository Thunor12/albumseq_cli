# albumseq_cli

**Mission:**  
`albumseq_cli` helps musicians, producers, and collectors sequence album tracks for physical media (like vinyl, cassette, or CD) by optimizing track order and side splits according to user-defined constraints and media limitations.

---

## Features

- **Tracklist Management:** Add, replace, and view named tracklists.
- **Medium Support:** Define physical media (vinyl, cassette, CD, etc.) with sides and duration limits.
- **Constraint System:** Specify rules (e.g., adjacency, position, side) to guide sequencing.
- **Optimized Proposals:** Generate top-scoring track orders that fit your medium and constraints.
- **Context Persistence:** All data is saved to a context file (default: `context.json`).

---

## Installation

```sh
git clone https://github.com/yourusername/albumseq_cli.git
cd albumseq_cli
cargo build --release
```

---

## Usage

Initialize a new context file:

```sh
albumseq_cli init
```

Add a tracklist:

```sh
albumseq_cli add-tracklist --name "MyPlaylist" --tracks "Intro:3:30" "Song 1:4.2"
```

Add a medium (e.g., vinyl):

```sh
albumseq_cli add-medium --name "Vinyl" --sides 2 --max-duration 20:00
```

Add a constraint (e.g., require "Intro" at position 0):

```sh
albumseq_cli add-constraint --kind atpos --args Intro 0 --weight 50
```

Propose top 10 sequences for a tracklist and medium:

```sh
albumseq_cli propose --tracklist "MyPlaylist" --medium "Vinyl" --count 10
```

Show all context data:

```sh
albumseq_cli show
```

---

## Command Reference

- `init`  
  Initialize a new context file.

- `add-tracklist`  
  Add or replace a named tracklist.  
  _Example:_  
  `albumseq_cli add-tracklist --name "My Album" --tracks "Song1:3:45" "Song2:4:10"`

- `add-medium`  
  Add or replace a named medium.  
  _Example:_  
  `albumseq_cli add-medium --name "Vinyl" --sides 2 --max-duration 22:00`

- `add-constraint`  
  Add a constraint to the context.  
  _Example:_  
  `albumseq_cli add-constraint --kind adjacent --args "Song1" "Song2" --weight 2`

- `remove-constraint`  
  Remove a constraint by index.  
  _Example:_  
  `albumseq_cli remove-constraint --index 0`

- `show`  
  Show the current context or filtered parts of it.  
  _Example:_  
  `albumseq_cli show --filter tracklists`

- `propose`  
  Propose top scoring tracklist permutations for a tracklist & medium.  
  _Example:_  
  `albumseq_cli propose --tracklist "My Album" --medium "Vinyl" --count 10 --min-score 5`

---

## Tips

- Use `--help` with any command for detailed options, e.g.:
  ```sh
  albumseq_cli add-tracklist --help
  ```
- The context file is `context.json` by default, but you can specify another with `--context`.

---

## License

GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007

---

## Contributing

Pull requests and suggestions are welcome! Please open an issue or PR