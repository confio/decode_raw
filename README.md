# decode_raw

A protobuf debugging tool – `protoc --decode_raw` on steroids.

## Installation

Install from crates.io:

```sh
cargo install decode_raw
```

Update to latest version:

```sh
cargo install --force decode_raw
```

## Basic usage

decode_raw reads serialized protobuf from STDIN and prints it.

**From pipe**

```
$ echo 08bf99bfb4e502120a4a616e6520536d697468 | xxd -r -p | decode_raw
1: 95941545151
2: (10 bytes) 'Jane Smith'
```

**From file**

```
decode_raw < docs/person.bin
1: 1021211
2: (8 bytes) 'John Doe'
3 {
· 1: 959435311
· 2: (11 bytes) 'Susanne Doe'
}
3 {
· 1: 81154811
· 2: (9 bytes) 'Mac Smith'
· 3 {
· · 1: 95941545151
· · 2: (10 bytes) 'Jane Smith'
· }
}
```

## Goals & non-goals

decode_raw should

- Make Simon happy when debugging protobuf
- Be trivial to get started for users of `protoc --decode_raw`
- Support proto3

but it does not intend to:

- Provide stable outputs for scripting
- Become a performance winner
- Help with broken protobuf serialization. Those will be considered raw app level bytes.
- Support proto2

## License

Apache 2.0, see [LICENSE](./LICENSE) and [NOTICE](./NOTICE).
