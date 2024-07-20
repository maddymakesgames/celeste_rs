# celeste_rs/0.4.0 - 2024-07-20
- Fix bug where `AmbienceVolume` can be non-null

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
