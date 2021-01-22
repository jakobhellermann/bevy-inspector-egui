# Changelog

## Version 0.2.0 (unreleased)

### Added
- [impl Inspectable for the remaining number types](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b072035c1f14baf189274e9a166f2bae1adf2f70)
- [impl Inspectable Mat{3,4}](https://github.com/jakobhellermann/bevy-inspector-egui/commit/070e2a45dec945a83e42ea268cc384cf6857a7bc)
- [`ReflectedUI` wrapper type for automatically figuring out how to display a type based on the `Reflect` impl](https://github.com/jakobhellermann/bevy-inspector-egui/commit/caf1c31c25349d29011d5772bba7eb2709879eb5)
  ```rust
  #[derive(Inspectable, Default, Debug)]
  struct Data { timer: ReflectedUI<Timer> }
  ```
- [allow multiple inspector windows](https://github.com/jakobhellermann/bevy-inspector-egui/commit/980de51e181fe486ac74312474c33a34e0a77293)

### Changed
- [rename NumberAttributes::step to speed](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b2aeb1735bdf0b5d8d68386e22f1e73437cbf733)
- [use the relecant number type to specify min and max of the NumberAttributes instead of always f64](https://github.com/jakobhellermann/bevy-inspector-egui/commit/28fa524390874026b0087b5bfac18b5e15ad1eec)
- [simplify inspectable trait, rename FieldOptions to Options](https://github.com/jakobhellermann/bevy-inspector-egui/commit/af032d42ac7f816ccc17a79eaf2e19f3768bb968)
  The trait now looks like this:
  ```rust
  trait Inspectable {
      type Attributes: Default;
      fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context);
  }
  ```
- [try to convert Attributes using From::from](https://github.com/jakobhellermann/bevy-inspector-egui/commit/43e84d885ca080a6f2e62bbcfb396f27f92237fd)

### Fixed
- [normalize quaternion in `Transform` so that it stays a valid rotation](https://github.com/jakobhellermann/bevy-inspector-egui/commit/48fce89f7692408bad4841b126f7e68d8995fffd)
- [disallow negative scale](https://github.com/jakobhellermann/bevy-inspector-egui/commit/c1ffc5d8898d2db882d60f88f8010c6121ca41ad)

## Version 0.1.0 (2021-01-18)

### Added
- first version with `Inspectable` support for number types `u8`, `i32` and `f{32,64}`, `String` and `Vec<T>` and bevy's `Color`, `Transform`, `Quat`, `Vec{2,3,4}`
- derive `Inspectable` for unit enums (displays a dropdown)
- derive `Inspectable` for struct
- `#[inspectable(label = x, collapse)]` builtins for struct derives
- `nightly` features for array `Inspectable` impls