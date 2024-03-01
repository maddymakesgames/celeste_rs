# celeste_rs gui
You can find a web version of the gui editor [here](https://maddymakesgames.github.io/celeste_rs/).<br>

## Design
We separate the app into 3 states, though in reality only 1 of them really matters.<br>
We have the `Startup` state, which just transitions immediately into the `MainMenu` state.<br>
The `MainMenu` state displays a button to request a save file, and handles reading in the save file and then transitions to the `Editor` state.<br>
The editor state is internally separated into different sections with each section having its own render method.
## Rational
This app uses the [egui]() gui library. This is mainly due to it being something I've used before and intermediate mode being quite good for what is essentially just editing a bunch of fields on a struct. This does mean our web app is not the best. Personally I prefer native applications hence why I am okay with this, but I do understand others may perfer a web app. This was the main reason I separated the reading / writing of save files and the gui logic into two separate crates, someone looking to make a more proper web app can just include the `celeste_rs` library and use that for the i/o logic.