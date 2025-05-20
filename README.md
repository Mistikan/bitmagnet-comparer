# bitmagnet-comparer
Utility for extracting hashes from bitmagnet and outputting them to the console. 

# Example usage
## Only output
Date is optional, needed for rescanning.
```sh
bitmagnet-comparer --bitmagnet-postgresql-url postgresql://postgres:postgres@localhost:7432/bitmagnet single test.avi 956716040
bitmagnet-comparer --bitmagnet-postgresql-url postgresql://postgres:postgres@localhost:7432/bitmagnet single test.avi 956716040 2025-12-10T00:00:00+03:00
```

Output:
```sh
cb879c48f3dabc747b4fbd6152fb5fa835fb63cf
```

## Find-torrent-data database
```sh
bitmagnet-comparer \
  --bitmagnet-postgresql-url postgresql://postgres:postgres@localhost:7432/bitmagnet \
  find-torrent-data-postgresql postgresql://postgres:postgres@localhost:7432/find_torrent_data
```

## Debug
```sh
RUST_LOG=info bitmagnet-comparer --bitmagnet-postgresql-url postgresql://postgres:postgres@localhost:7432/bitmagnet single test.avi 956716040
```

Output:
```
postgres:postgres@localhost:7432/bitmagnet single test.avi 956716040
{"level":"Info","ts":1747593305765,"msg":"Get input files"}
{"level":"Info","ts":1747593305765,"msg":"Search hashs"}
{"level":"Info","ts":1747593305765,"msg":"Print hashs"}
{"level":"Info","ts":1747593305765,"msg":"Get hash for file: File { path: \"test.avi\", size: 956716040 }"}
{"level":"Info","ts":1747593305765,"msg":"Connect db"}
{"level":"Info","ts":1747593305818,"msg":"Extension: Some(\"avi\"), Size: 956716040"}
cb879c48f3dabc747b4fbd6152fb5fa835fb63cf
```

## Qbittorrent
**IMPORTANT**: It is recommended to set in Qbittorrent `Settings -> Downloads -> Torrent stop condition: "Metadata received"`, otherwise automatic download will start.

```sh
bitmagnet-comparer --bitmagnet-postgresql-url postgresql://postgres:postgres@localhost:7432/bitmagnet single test.avi 956716040 | xargs -t -I {} qbt torrent add url --tag only-metadata --url http://localhost:8080 magnet:?xt=urn:btih:{}
```

# Links
* [bitmagnet](https://github.com/bitmagnet-io/bitmagnet)
* [qbt](https://github.com/fedarovich/qbittorrent-cli)
