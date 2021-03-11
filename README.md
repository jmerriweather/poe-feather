# PoE-FeatherWing Driver

A no_std Rust driver for the [PoE FeatherWing by Silicognition LLC](https://www.crowdsupply.com/silicognition/poe-featherwing)

![PoE FeatherWing](poe-featherwing-front-back-01.jpg?raw=true)

## Feature Flags

All features are disabled by default.

* `defmt`: Passthrough to [w5500-hl].
* `std`: Passthrough to [w5500-hl].

## Examples

* [UDP Example with Adafruit nrf52840 Express](examples/udp)

## Related Crates

* [w5500-hl](https://github.com/newAM/w5500-hl-rs) - High level W5500 socket operations.
* [w5500-ll](https://github.com/newAM/w5500-ll-rs) - Low level W5500 register accessors.
