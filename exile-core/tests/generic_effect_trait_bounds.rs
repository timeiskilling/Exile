use std::fmt::Debug;

use exile_core::{
    effect::{EffectEntry, EffectOrigin},
    game::{Game, ModifierDefinitionIdentity},
};

struct TraitBoundGame;

struct TraitBoundModifierDefinition;

impl ModifierDefinitionIdentity for TraitBoundModifierDefinition {
    type Id = ();

    fn modifier_definition_id(&self) -> Self::Id {}
}

impl Game for TraitBoundGame {
    type ItemBase = ();
    type ItemState = ();

    type ModifierDefinitionId = ();
    type ModifierDefinition = TraitBoundModifierDefinition;

    type ModifierInstance = ();

    type Effect = u8;
    type EffectCondition = bool;
    type EffectSourceId = ();
}

fn assert_debug<T>()
where
    T: Debug,
{
}

fn assert_clone<T>()
where
    T: Clone,
{
}

fn assert_partial_eq<T>()
where
    T: PartialEq,
{
}

fn assert_eq<T>()
where
    T: Eq,
{
}

#[test]
fn effect_entry_traits_depend_on_associated_types() {
    assert_debug::<EffectEntry<TraitBoundGame>>();

    assert_partial_eq::<EffectEntry<TraitBoundGame>>();
}

#[test]
fn effect_origin_traits_depend_on_associated_types() {
    assert_debug::<EffectOrigin<TraitBoundGame>>();

    assert_clone::<EffectOrigin<TraitBoundGame>>();

    assert_partial_eq::<EffectOrigin<TraitBoundGame>>();

    assert_eq::<EffectOrigin<TraitBoundGame>>();
}
