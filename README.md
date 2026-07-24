# `enum-info`

## Overview

`enum-info` is a simple crate intended for the niche scenario where you need to
 know the number of variants an enum has, as well as having a list of each
 variant (for enums consisting of only unit variants).

## Example

Imagine making a game where you can select between multiple characters —
 in Rust, this selection would be represented by an enum.

In order to make a character select menu that can seamlessly handle the addition
 of new characters, you'd usually have to track all the members of the enum and
 manually update the size of some array; however, with this library, it's as
 simple as:

```rust
#[enum_info]
enum Characters {
	Katty,
	Jax
}

// ...

assert_eq!(Characters::variant_count(), 2);
assert_eq!(Characters::VARIANTS, [Characters::Katty, Characters::Jax]);
```