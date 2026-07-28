use std::{cell::Cell, rc::Rc};

use exile_core::{
    effect::{
        BuildEffectCollector, EffectCollection, ItemEffectCollectionError, ItemEffectCollector,
        ModifierEffectResolver,
    },
    game::Game,
    item::{ItemInstance, ModifierDefinitionProvider, Validated},
};

use crate::support::{
    TestGame, TestModifierDefinitionProvider, TestModifierEffectResolver, TestPassiveNode,
};

pub struct TestBuild {
    items: Vec<ItemInstance<TestGame, Validated>>,
    passive_nodes: Vec<TestPassiveNode>,
}

impl TestBuild {
    pub fn new(
        items: Vec<ItemInstance<TestGame, Validated>>,
        passive_nodes: Vec<TestPassiveNode>,
    ) -> Self {
        Self {
            items,
            passive_nodes,
        }
    }

    pub fn items(&self) -> &[ItemInstance<TestGame, Validated>] {
        &self.items
    }

    pub fn passive_nodes(&self) -> &[TestPassiveNode] {
        &self.passive_nodes
    }
}

pub struct TestBuildEffectCollector<'a> {
    item_collector:
        ItemEffectCollector<'a, TestModifierDefinitionProvider, TestModifierEffectResolver>,
}

impl<'a> TestBuildEffectCollector<'a> {
    pub fn new(
        definitions: &'a TestModifierDefinitionProvider,
        resolver: &'a TestModifierEffectResolver,
    ) -> Self {
        Self {
            item_collector: ItemEffectCollector::new(definitions, resolver),
        }
    }
}

impl BuildEffectCollector<TestGame> for TestBuildEffectCollector<'_> {
    type Build = TestBuild;

    type Error = ItemEffectCollectionError<
        <TestModifierDefinitionProvider as ModifierDefinitionProvider<TestGame>>::Error,
        <TestModifierEffectResolver as ModifierEffectResolver<TestGame>>::Error,
    >;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<TestGame>, Self::Error> {
        let mut effects = EffectCollection::<TestGame>::new();

        effects.collect_from_items(&self.item_collector, build.items().iter())?;

        effects.collect_from_sources(build.passive_nodes().iter());

        Ok(effects)
    }
}

pub struct CountingBuildEffectCollector<BC> {
    inner: BC,
    calculation_count: Rc<Cell<usize>>,
}

impl<BC> CountingBuildEffectCollector<BC> {
    pub fn new(inner: BC) -> (Self, Rc<Cell<usize>>) {
        let calculation_count = Rc::new(Cell::new(0));

        let collector = Self {
            inner,
            calculation_count: Rc::clone(&calculation_count),
        };

        (collector, calculation_count)
    }
}

impl<G, BC> BuildEffectCollector<G> for CountingBuildEffectCollector<BC>
where
    G: Game,
    BC: BuildEffectCollector<G>,
{
    type Build = BC::Build;
    type Error = BC::Error;

    fn collect_effects(&self, build: &Self::Build) -> Result<EffectCollection<G>, Self::Error> {
        let next_count = self
            .calculation_count
            .get()
            .checked_add(1)
            .expect("build calculation count overflow");

        self.calculation_count.set(next_count);

        self.inner.collect_effects(build)
    }
}
