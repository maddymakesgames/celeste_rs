# celeste_rs/0.2.1 - 2024-03-28
- Fixed serialization issue where options were being serialized when they were `None`
- Make `SaveData::merge_data` better by merging: total strawberry count, assists, and unlocked areas. 
  - This includes a rudimentary merging strategy for total golden strawberries, but this is not that good right now 
- Fix the vanilla sid constants having farewell be labeled differently than it is in a modded environment

# celeste_rs/0.2.0 - 2024-03-27
- Fixed issue where saves would fail to load due to exported saves not importing the right xml extensions.
- Added xml version headers to output files
