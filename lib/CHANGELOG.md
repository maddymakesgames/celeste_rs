# celeste_rs/0.5.0 - 2024-08-29
- Add `Trigger` trait and a corresponding derive macro.
- Add `MapWriter` for actually writing the maps back to binary.
- Fix various representations of vanilla elements.
- Add support for all (used) vanilla entities and triggers. Found in the `vanilla_triggers` and `vanilla_entities` modules.
    - Vanilla maps can be read in, modified, and written out and be loaded by the vanilla game now.
- Add or update documentation for various structs.

# celeste_rs/0.4.0 - 2024-07-20
- Fix bug where `AmbienceVolume` can be non-null
- Start adding support for reading & writing maps
    - Map parsing and encoding can be accesssed through the `MapManager` struct
    - Any unrecognized elements found will be left as a `RawMapElement`
    - Any heterogenous arrays will store `DynMapElement`s which can be downcast into actual structs by checking against the element name.
    - You can add support for new elements by implementing `MapElement` for a struct and then using `MapManager::add_parser`
    - We also provide a derive macro for `MapElement` and `Entity`. Look at the implementations in `maps/elements` to see how to use them. 

# celeste_rs/0.3.0 - 2024-06-04
- Add support for `modsave`, `modsession`, and `modsetting` file parsing
- Add support for `x-AurorasAdditions-modsave` files
- Add support for `x-CollabUtils2-modsave` files
- Add support for generic YAML files
- Fix deserialization issue for Flags with an `xsi:nil` attribute

# celeste_rs/0.2.1 - 2024-03-28
- Fixed serialization issue where options were being serialized when they were `None`
- Make `SaveData::merge_data` better by merging: total strawberry count, assists, and unlocked areas. 
  - This includes a rudimentary merging strategy for total golden strawberries, but this is not that good right now 
- Fix the vanilla sid constants having farewell be labeled differently than it is in a modded environment

# celeste_rs/0.2.0 - 2024-03-27
- Fixed issue where saves would fail to load due to exported saves not importing the right xml extensions.
- Added xml version headers to output files
