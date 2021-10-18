use bevy_inspector_egui::Inspectable;
use std::marker::PhantomData;

struct Marker;
struct HasAssoc;
impl WithAssoc for HasAssoc {
    type AssocType = f32;
}

trait WithAssoc {
    type AssocType;
}

#[derive(Inspectable)]
#[inspectable(override_where_clause = "P::AssocType: Inspectable")]
struct Struct<M, P: WithAssoc> {
    x: f32,
    assoc: P::AssocType,
    #[inspectable(ignore)]
    m: PhantomData<M>,
}

#[derive(Inspectable)]
#[inspectable(override_where_clause = "")]
enum Enum<M> {
    A(f32),
    B(#[inspectable(ignore)] PhantomData<M>),
}

fn check<T: Inspectable>() {}

fn main() {
    check::<Struct<Marker, HasAssoc>>();
    check::<Enum<Marker>>();
}
