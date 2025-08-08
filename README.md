# albumseq_cli

```sh
cargo run -- add-tracklist -n "MyPlaylist" -t "Intro:3:30" -t "Song 1:4.2"
```

```sh
cargo run -- add-medium -n "Vinyl" -s 2 -m 20:00
```

```sh
cargo run -- add-constraint -k atpos -a Intro -a 0 -w 50
```

```sh
cargo run -- propose -t "MyAlbum" -m "Vinyl" -c 10
```