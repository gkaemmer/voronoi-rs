## Voronoi diagrams in Rust

This is a performant, but work-in-progress implementation of Fortune's Algorithm in Rust. It uses a custom red-black tree (based on [rust-redblack](https://github.com/gkaemmer/rust-redblack)) and a custom priority heap. They both support "handles", meaning that the calling code can reference items deep in the trees.

### TODO

- More examples
- Better API
- More return types other than edges (polygons)
