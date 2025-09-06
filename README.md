# bevy_control
Utility components for controlling entities in [Bevy](https://bevyengine.org).

## Features
- 3D Camera that can dynamically switch between first person and third person.
- 2D Camera that can follow an entity and will zoom in or out to a specified threshold.
- Both cameras can have transformation handled with smooth interpolation

## Goals
- Simplicity: components should just work by without issue.
- Flexibility: components should be modular, such that they can be used in any combination.
- Minimal: using the library should require as little boilerplate as possible.
- Extensibility: components shouldn't prevent the user from overriding and extending the behavior of entities.

## Todo
- Implement character controller.

## License
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)
