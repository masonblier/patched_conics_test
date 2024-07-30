# patched_conics_test

Patched conics ui test. References [Orbital Mechanics Notes](https://orbital-mechanics.space/intro.html). Uses the [Bevy engine](https://bevyengine.org/). Based on the [Bevy Game Template](https://github.com/NiklasEi/bevy_game_template).

Work in progress! Todo:
- Refactor moon parameters into config
- Various precision bugs, remove nudge factor
- Support three levels of heirarchy (sun -> planets -> moons)
- Reorganize as importable library, only dependency should be glam
- More tests, multiple examples

![patched_conics_demo_03](https://github.com/masonblier/patched_conics_test/assets/677787/1b7bfd90-a8ee-4d4c-b88d-b2aaf4b76885)

# Running from source

* Start the native app: `cargo run`

# License

This project is licensed under [CC0 1.0 Universal](LICENSE). Some assets under respective licenses.
