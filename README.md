# butterfly

This crate contains LibAFL components that are useful when fuzzing network applications:
- `PacketBasedInput`: A type implementing `Input` that is a vector of packets + mutators
  to mutate the packet vector
- `TokenStream`: If the network protocol is text-based, this type offers a representation of
  text as a stream of `TextToken`'s that can be meaningfully mutated

