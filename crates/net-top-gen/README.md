# net-top-gen

Generates network topologies using simdash-state.

Can specify nodes using nested json like:

```json
[
  {
    type: "nodes",
    number: 100,
    links: [
      {
        type: "links",
        number: 1,
        destination: "random"
      }
    ]
  }
]
```