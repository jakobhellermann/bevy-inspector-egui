# Changelog

## Version 0.2.0 (unreleased)

### Changed
- [rename NumberAttributes::step to speed](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b2aeb1735bdf0b5d8d68386e22f1e73437cbf733)
- [use the relecant number type to specify min and max of the NumberAttributes instead of always f64](https://github.com/jakobhellermann/bevy-inspector-egui/commit/28fa524390874026b0087b5bfac18b5e15ad1eec)
- [impl Inspectable for the remaining number types](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b072035c1f14baf189274e9a166f2bae1adf2f70)
- [simplify inspectable trait, rename FieldOptions to Options](https://github.com/jakobhellermann/bevy-inspector-egui/commit/af032d42ac7f816ccc17a79eaf2e19f3768bb968)
  The trait now looks like this:
  ```rust
  trait Inspectable {
      type Attributes: Default;
      fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes);
  }
  ```

## Version 0.1.0 (2021-01-18)

### Added
- first version with `Inspectable` support for number types `u8`, `i32` and `f{32,64}`, `String` and `Vec<T>` and bevy's `Color`, `Transform`, `Quat`, `Vec{2,3,4}`
- derive `Inspectable` for unit enums (displays a dropdown)
- derive `Inspectable` for struct
- `#[inspectable(label = x, collapse)]` builtins for struct derives
- `nightly` features for array `Inspectable` impls