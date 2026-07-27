# `exile-core` Library Architecture & Usage Guide

## 1. High-Level Architecture & Core Flow

The `exile-core` library is a generic, trait-driven domain engine for Action RPG systems. Its purpose is to model items, modifier definitions, rolled modifier instances, effect sources, conditional effects, deterministic execution order, final stat calculation, and current-versus-candidate build comparison without hardcoding the rules of a particular game.

The crate defines the calculation process, while the consuming game defines the data and semantics. A concrete integration supplies its own item bases, item state, modifier identifiers, modifier definitions, modifier instances, effects, conditions, source identifiers, planning rules, accumulator, final output, and comparison model.

The complete flow begins with domain state and ends with either a finalized output or a comparison between the current build and a hypothetical candidate.

```text
ItemInstance<Unvalidated>
    -> ItemValidator
    -> ItemInstance<Validated>
    -> ItemEffectCollector or BuildEffectCollector
    -> EffectCollection
    -> EffectCollectionEvaluator
    -> ActiveEffectCollection
    -> EffectPlanner
    -> EffectExecutionPlan
    -> EffectCollectionApplier
    -> EffectAccumulatorFinalizer
    -> Final Output
```

At the build level, this process is wrapped by `BuildCalculationRunner`. Stateful applications can then place the runner inside `BuildCalculationCore`, which owns the current build, evaluation context, calculation input, accumulator factory, cached baseline, and candidate-comparison lifecycle.

The crate therefore has three distinct architectural layers. The item layer represents and validates domain state. The effect layer converts domain state into sourced and conditionally active effects. The calculation layer orders those effects, applies them to an accumulator, finalizes the output, caches the current result, and compares candidate builds.

---

## 2. The `Game`

The `Game` trait is the central type registry for a concrete game implementation. It does not perform calculation itself. Instead, it connects all generic subsystems to the concrete types used by the consuming project.

```rust
pub trait ModifierDefinitionIdentity {
    type Id;

    fn modifier_definition_id(&self) -> Self::Id;
}

pub trait Game {
    type ItemBase;
    type ItemState;

    type ModifierDefinitionId;
    type ModifierDefinition:
        ModifierDefinitionIdentity<Id = Self::ModifierDefinitionId>;

    type ModifierInstance;

    type Effect;
    type EffectCondition;
    type EffectSourceId;
}
```

`ItemBase` represents the base identity or category of an item. It can describe concepts such as a sword base, a pair of boots, a ring, or any other game-specific base type.

`ItemState` represents the current mutable state of an item. Typical examples include item level, rarity, quality, corruption state, or any other state that belongs to the item rather than to an individual modifier.

`ModifierDefinitionId` identifies a modifier rule. `ModifierDefinition` stores the complete rule and must implement `ModifierDefinitionIdentity` so the identifier can be obtained from the definition itself. `ModifierInstance` stores the concrete rolled or configured value attached to an item.

`Effect` is the runtime operation consumed by the calculation pipeline. `EffectCondition` represents a condition that may enable or disable an effect during evaluation. `EffectSourceId` identifies a non-item effect source when the source is recorded in `EffectOrigin::Source`.

A concrete game implementation may look like this:

```rust
pub struct MyGame;

impl Game for MyGame {
    type ItemBase = MyItemBase;
    type ItemState = MyItemState;

    type ModifierDefinitionId = MyModifierDefinitionId;
    type ModifierDefinition = MyModifierDefinition;
    type ModifierInstance = MyModifierInstance;

    type Effect = MyEffect;
    type EffectCondition = MyEffectCondition;
    type EffectSourceId = MyEffectSourceId;
}
```

`Game` should remain a type-level contract. Runtime services such as databases, caches, parsers, providers, collectors, or network clients belong in their own structs and traits rather than inside the `Game` type.

---

## 3. Item Model, Editing, Parsing, and Validation

### `ItemInstance`

`ItemInstance<G, ValidationState>` is the main runtime representation of an item. It owns the item base, item state, stored modifiers, the next modifier-instance identifier, a revision counter, and a typestate marker.

The default validation state is `Unvalidated`.

```rust
pub struct ItemInstance<G: Game, ValidationState = Unvalidated> {
    base: G::ItemBase,
    state: G::ItemState,
    modifiers: Vec<
        StoredModifier<
            G::ModifierDefinitionId,
            G::ModifierInstance,
        >,
    >,
    next_modifier_id: u64,
    revision: u64,
    validation_state: PhantomData<ValidationState>,
}
```

The item model separates the identity of a modifier rule from the identity of a stored modifier instance. `ModifierDefinitionId` identifies the rule, while `ModifierInstanceId` identifies one concrete modifier stored in one item. `StoredModifier` combines the instance ID, definition ID, and rolled modifier value.

The common read API is available for both validation states. Callers can read the base, state, modifier slice, individual modifier values, complete stored modifiers, and the current revision without mutating the item.

### Typestate validation

An item created with `ItemInstance::new` or `ItemInstance::from_parts` is unvalidated. The only public transition to `Validated` is the `validate` method.

```rust
let validated_item = item.validate(&validator)?;
```

Validation consumes the unvalidated item. On success, it returns `ItemInstance<G, Validated>`. On failure, it returns `ItemValidationFailure<G, E>`, which preserves both the original unvalidated item and the validation error.

```rust
match item.validate(&validator) {
    Ok(validated) => {
        use_validated_item(validated);
    }
    Err(failure) => {
        let (item, error) = failure.into_parts();
        handle_invalid_item(item, error);
    }
}
```

This design prevents invalid external data from being silently promoted while still allowing the caller to recover and inspect the original item after a failed validation attempt.

### Item editing

`ItemEditor<R>` owns an `ItemRule` implementation and applies controlled mutations to `ItemInstance<G, Unvalidated>`. The current API provides methods for adding, removing, and replacing modifiers, as well as replacing the item state.

Before committing a mutation, the editor delegates to the corresponding rule method. A successful mutation increments the item revision. Modifier replacement keeps the same `ModifierInstanceId`, while removal and replacement return the previous modifier value.

```rust
let editor = ItemEditor::new(MyItemRule);

let id = editor.add_modifier(
    &mut item,
    &definition,
    modifier,
)?;

let previous = editor.replace_modifier(
    &mut item,
    id,
    &replacement_definition,
    replacement_modifier,
)?;
```

The current implementation deliberately edits unvalidated items. A caller that needs to modify a validated item must first convert it back with `into_unvalidated`, perform the edits, and validate it again.

```rust
let mut item = validated_item.into_unvalidated();

editor.replace_state(&mut item, new_state)?;

let validated_item = item.validate(&validator)?;
```

### Item rules

`ItemRule<G>` defines whether a requested mutation is legal. It receives the current unvalidated item and the proposed change, then returns either `Ok(())` or a game-specific error.

The rule is responsible for domain constraints such as incompatible modifier groups, affix limits, state-dependent restrictions, or any other mutation rule. The editor remains responsible for committing the mutation only after validation succeeds.

### Modifier definitions and validation

`ModifierDefinitionProvider<G>` resolves a borrowed modifier definition from a definition identifier.

```rust
fn definition(
    &self,
    id: &G::ModifierDefinitionId,
) -> Result<&G::ModifierDefinition, Self::Error>;
```

`ModifierValidator<G>` validates one modifier definition and modifier instance in the context of an item. `ItemValidator<G>` validates the complete unvalidated item.

This separation allows a project to compose validation from a definition registry, per-modifier validation, and complete-item invariants without forcing the core crate to know how definitions are stored.

### Parsing

`ItemTextParser<G>` converts external text into `ItemInstance<G, Unvalidated>`. `ModifierTextParser<G>` attempts to parse one line and returns an optional `ModifierPair<G>`, where the pair contains a definition ID and modifier instance.

Parsing and validation are intentionally separate. A parser describes what the input contains, while a validator decides whether that parsed state is acceptable.

```text
External Text
    -> ItemTextParser
    -> ItemInstance<Unvalidated>
    -> ItemValidator
    -> ItemInstance<Validated>
```

---

## 4. Effect Model and Provenance

### `EffectEntry`

`EffectEntry<G>` stores one effect and an optional condition. The crate provides explicit constructors for unconditional and conditional effects.

```rust
let unconditional =
    EffectEntry::<MyGame>::unconditional(effect);

let conditional =
    EffectEntry::<MyGame>::conditional(effect, condition);
```

`EffectEntry::is_active` contains the basic activation rule. An unconditional effect is always active. A conditional effect delegates to an `EffectConditionEvaluator<G>`.

### `EffectOrigin`

Every collected calculation effect can retain information about where it came from. `EffectOrigin<G>` currently has three variants.

```rust
pub enum EffectOrigin<G>
where
    G: Game,
{
    ItemModifier {
        modifier_instance_id: ModifierInstanceId,
        definition_id: G::ModifierDefinitionId,
    },

    ModifierDefinition {
        definition_id: G::ModifierDefinitionId,
    },

    Source(G::EffectSourceId),
}
```

`ItemModifier` is used when an effect was resolved from a concrete modifier stored on a validated item. `ModifierDefinition` is used when a definition and modifier instance are resolved directly without a concrete item. `Source` is used for objects implementing `EffectSource<G>`.

This provenance is preserved through evaluation, planning, rejection reporting, and detailed calculation output.

### `SourcedEffectEntry`

`SourcedEffectEntry<G>` combines an `EffectEntry<G>` with an `EffectOrigin<G>`. It is the canonical runtime unit stored in an `EffectCollection`.

The type exposes the original entry, effect, condition, origin, activation check, and an owned `into_parts` conversion. The effect and its provenance therefore remain associated throughout the pipeline.

### `EffectCollection`

`EffectCollection<G>` owns a `Vec<SourcedEffectEntry<G>>`. It supports direct iteration and can collect effects from generic sources, modifier resolution, one validated item, or multiple validated items.

```rust
let mut effects = EffectCollection::<MyGame>::new();

effects.collect_from_source(&source);

effects.collect_from_modifier(
    &resolver,
    &definition,
    &modifier,
)?;

effects.collect_from_item(
    &item_collector,
    &validated_item,
)?;
```

Collection from an item is atomic with respect to the destination collection. `ItemEffectCollector::collect` first builds a local vector of sourced entries. The destination collection is extended only after the complete item succeeds.

Collection from multiple items is also atomic. `collect_from_items` accumulates results in a temporary `EffectCollection` and extends the destination only after all items have been collected successfully.

---

## 5. Effect Sources and Item Effect Resolution

### `EffectSource`

`EffectSource<G>` represents an already available domain object that can directly produce effects.

```rust
pub trait EffectSource<G>
where
    G: Game,
{
    fn effect_source_id(&self) -> G::EffectSourceId;

    fn collect_effects(&self) -> Vec<EffectEntry<G>>;
}
```

The source returns its game-specific source identifier and a vector of effect entries. `EffectCollection::collect_from_source` wraps each returned entry in `EffectOrigin::Source`.

The trait is currently infallible. Loading or lookup failures should occur before the source is passed into collection, typically through a provider.

### `PassiveNodeProvider`

`PassiveNodeProvider<G>` resolves a borrowed node that implements `EffectSource<G>`.

```rust
pub trait PassiveNodeProvider<G>
where
    G: Game,
{
    type Id;
    type Node: EffectSource<G>;
    type Error;

    fn node(
        &self,
        id: &Self::Id,
    ) -> Result<&Self::Node, Self::Error>;
}
```

`PassiveNodeProvider::Id` is the lookup key used by the provider. It is independent from `G::EffectSourceId`, although a concrete game may choose to use the same underlying type for both.

### `ModifierEffectResolver`

`ModifierEffectResolver<G>` translates a modifier definition and modifier instance into zero or more raw `EffectEntry<G>` values.

```rust
pub trait ModifierEffectResolver<G>
where
    G: Game,
{
    type Error;

    fn resolve_modifier_effects(
        &self,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<Vec<EffectEntry<G>>, Self::Error>;
}
```

The resolver does not attach provenance. The caller decides whether the resulting origin is a concrete item modifier or a direct modifier definition.

### `ItemEffectCollector`

`ItemEffectCollector<'a, P, R>` borrows a modifier-definition provider and a modifier-effect resolver. It accepts `ItemInstance<G, Validated>` and returns sourced entries.

For every stored modifier, the collector obtains the definition, resolves the definition and rolled instance into effect entries, and assigns an `EffectOrigin::ItemModifier` containing the modifier-instance ID and cloned definition ID.

The collector distinguishes provider failures from resolver failures through `ItemEffectCollectionError<DefinitionError, ResolveError>`.

---

## 6. Conditional Effect Evaluation

`EffectConditionEvaluator<G>` defines the context type and the game-specific logic for evaluating one effect condition.

```rust
pub trait EffectConditionEvaluator<G>
where
    G: Game,
{
    type Context;
    type Error;

    fn evaluate_condition(
        &self,
        condition: &G::EffectCondition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error>;
}
```

`EffectCollectionEvaluator<E>` owns a concrete condition evaluator. Its `collect_active` method traverses an `EffectCollection`, evaluates every conditional sourced entry, and builds an `ActiveEffectCollection<'a, G>` containing references to the active entries.

```rust
let evaluator =
    EffectCollectionEvaluator::new(MyConditionEvaluator);

let active_effects =
    evaluator.collect_active(&effects, &context)?;
```

`ActiveEffectCollection` owns only a vector of references. It does not clone or own the sourced entries. Its lifetime is therefore tied to the original `EffectCollection`.

The type provides iteration over complete sourced entries, effects only, or origins only. The constructor is crate-private, which keeps active collections on trusted evaluation paths.

---

## 7. Planning and Execution Order

### `EffectPlanner`

`EffectPlanner<G>` converts an active effect collection into an execution plan.

```rust
pub trait EffectPlanner<G>
where
    G: Game,
{
    type Error;

    fn plan<'a>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
    ) -> Result<EffectExecutionPlan<'a, G>, Self::Error>;
}
```

The plan borrows the same sourced entries as the active collection. A custom game may implement the planner directly, or it may use the generic `EffectExecutionPlanner<P>` together with an `EffectPlanningPolicy<G>`.

### `EffectPlanningPolicy`

`EffectPlanningPolicy<G>` defines the semantic information required by the generic planner.

```rust
pub trait EffectPlanningPolicy<G>
where
    G: Game,
{
    type Phase: Ord;
    type Priority: Ord;
    type ConflictKey: Clone + Eq + Hash;
    type SelectionKey: Eq + Hash;

    fn phase(&self, effect: &G::Effect) -> Self::Phase;

    fn priority(&self, effect: &G::Effect) -> Self::Priority;

    fn conflict_key(
        &self,
        effect: &G::Effect,
    ) -> Option<Self::ConflictKey>;

    fn selection_key(
        &self,
        effect: &G::Effect,
    ) -> Option<Self::SelectionKey>;

    fn prefers(
        &self,
        candidate: &G::Effect,
        current: &G::Effect,
    ) -> bool;
}
```

Phase and priority define deterministic execution order. Conflict keys identify effects that are not allowed to coexist. Selection keys identify groups where only one effect should survive. `prefers` decides whether a new candidate should replace the current winner in a selection group.

Policy methods are expected to behave deterministically for the same effect and policy state. The generic planner may query the same selection key more than once during planning.

### `EffectExecutionPlanner`

`EffectExecutionPlanner<P>` performs planning in three stages. It first builds an ordered preliminary plan using phase, priority, and original collection position. It then validates exclusive conflict keys. Finally, it selects one winner for every selection group and records the rejected effects.

The original collection index is used as the final ordering key. Effects with the same phase and priority therefore remain deterministic.

When two effects share the same conflict key, planning fails with `EffectExecutionPlanValidationError`. The error owns the conflict key and cloned origins of the first and second conflicting effects.

Winner selection is performed in two passes. The planner first determines the final winner index for each selection key. It then constructs the selected entry list and creates `EffectSelectionRejection` records for every losing entry. Every rejection points to the final winner rather than an intermediate candidate.

### `EffectExecutionPlan`

`EffectExecutionPlan<'a, G>` contains ordered selected entries and selection-rejection metadata. It provides iteration over sourced entries and effects, as well as access to rejection records and their count.

Although `EffectExecutionPlan::build` creates an ordered plan, the normal complete planning path is `EffectPlanner::plan`, because the planner also validates conflicts and performs winner selection.

---

## 8. Accumulator Creation, Effect Application, and Finalization

### `EffectAccumulatorFactory`

`EffectAccumulatorFactory` creates a fresh accumulator from a calculation input.

```rust
pub trait EffectAccumulatorFactory {
    type Input;
    type Accumulator;
    type Error;

    fn create(
        &self,
        input: &Self::Input,
    ) -> Result<Self::Accumulator, Self::Error>;
}
```

The input can contain base character state, base skill values, or any other information required before effects are applied.

### `EffectApplier`

`EffectApplier<G>` applies one game effect to a mutable accumulator.

```rust
pub trait EffectApplier<G>
where
    G: Game,
{
    type Accumulator;
    type Error;

    fn apply_effect(
        &self,
        effect: &G::Effect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error>;
}
```

The applier owns the concrete arithmetic semantics of the game. Planning determines which effects are executed and in what order; the applier performs the actual mutation.

### `EffectCollectionApplier`

`EffectCollectionApplier<A>` owns a concrete effect applier and applies every effect in an `EffectExecutionPlan`.

```rust
for effect in plan.effects() {
    self.effect_applier
        .apply_effect(effect, accumulator)?;
}
```

The collection applier does not reorder effects or resolve conflicts. It trusts the execution plan and stops on the first application error.

### `EffectAccumulatorFinalizer`

`EffectAccumulatorFinalizer` consumes the completed accumulator and produces the finalized output.

```rust
pub trait EffectAccumulatorFinalizer {
    type Accumulator;
    type Output;
    type Error;

    fn finalize(
        &self,
        accumulator: Self::Accumulator,
    ) -> Result<Self::Output, Self::Error>;
}
```

The finalizer may calculate derived values, apply final caps, enforce output invariants, or convert an internal mutable structure into the immutable result returned to the caller.

---

## 9. `EffectCalculator` and Detailed Calculation Output

`EffectCalculator<A, F, P>` combines an `EffectCollectionApplier<A>`, a finalizer, and a planner.

```rust
let calculator = EffectCalculator::new(
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
);
```

The calculator exposes four calculation paths.

`calculate` accepts an active effect collection and an already created accumulator. It plans the effects, applies the plan, finalizes the accumulator, and returns only the finalized output.

`calculate_from_input` first asks an `EffectAccumulatorFactory` to create the accumulator, then performs the same planning, application, and finalization sequence.

`calculate_detailed` accepts an already created accumulator and returns `EffectCalculationOutput<'a, G, O>`, which contains both the finalized output and the execution plan.

`calculate_from_input_detailed` creates the accumulator from input and also returns the detailed output.

```rust
let output = calculator.calculate_from_input(
    &active_effects,
    &factory,
    &input,
)?;

let detailed = calculator.calculate_from_input_detailed(
    &active_effects,
    &factory,
    &input,
)?;
```

The standard calculation error distinguishes planning, application, and finalization failures. The input-based error adds accumulator creation as a separate stage.

```rust
pub enum EffectCalculationError<
    PlanError,
    ApplyError,
    FinalizeError,
> {
    Plan(PlanError),
    Apply(ApplyError),
    Finalize(FinalizeError),
}

pub enum EffectCalculationFromInputError<
    CreateError,
    PlanError,
    ApplyError,
    FinalizeError,
> {
    CreateAccumulator(CreateError),
    Plan(PlanError),
    Apply(ApplyError),
    Finalize(FinalizeError),
}
```

Because detailed output owns the execution plan, it borrows the sourced effects through the plan lifetime. Calling `into_output` discards the plan and returns the owned finalized output.

---

## 10. Build-Level Calculation

### `BuildEffectCollector`

`BuildEffectCollector<G>` is the boundary between a game-specific build model and the generic effect pipeline.

```rust
pub trait BuildEffectCollector<G>
where
    G: Game,
{
    type Build;
    type Error;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<G>, Self::Error>;
}
```

The build type is not defined by `exile-core`. A concrete application may include items, passive nodes, selected skills, gems, buffs, or any other source of effects.

### `BuildCalculationRunner`

`BuildCalculationRunner<BC, E, A, F, P>` combines a build collector, an effect collection evaluator, and an effect calculator.

```rust
let runner = BuildCalculationRunner::new(
    build_collector,
    evaluator,
    calculator,
);
```

`calculate_build` performs the complete stateless build calculation.

```text
Build
    -> BuildEffectCollector
    -> EffectCollection
    -> EffectCollectionEvaluator
    -> ActiveEffectCollection
    -> EffectCalculator
    -> Final Output
```

The runner exposes references to the build collector, evaluator, and calculator, and it can be consumed into those three components with `into_parts`.

`BuildCalculationError` preserves the stage where the build failed. Collection errors originate in the build collector, evaluation errors originate in the condition evaluator, and calculation errors contain the input-based calculator error.

---

## 11. Stateful Calculation with `BuildCalculationCore`

`BuildCalculationCore<G, BC, E, A, F, P, Factory, C>` is the high-level stateful API. It owns the current build, condition context, calculation input, accumulator factory, build calculation runner, comparison runner, internal generation, and optional cached baseline.

```rust
let mut core = BuildCalculationCore::new(
    current_build,
    context,
    input,
    factory,
    runner,
    comparison_runner,
);
```

### Current calculation and baseline

`calculate_current` always recalculates the current build. After a successful calculation, the result is stored in a `CalculationBaseline<CoreGeneration, Output>` and a reference to the cached output is returned.

A failed calculation does not insert a new baseline.

```rust
let current_output =
    core.calculate_current()?;
```

The private `ensure_baseline` method is used by candidate comparison. It calculates the current build only when no baseline exists. Repeated candidate comparisons can therefore reuse the cached current output while recalculating only the candidate.

### Managed mutation

The core provides `replace_build`, `replace_context`, and `replace_input`. Each replacement computes the next internal generation using checked arithmetic, swaps the owned value, and invalidates the cached baseline.

```rust
let previous_build =
    core.replace_build(updated_build)?;
```

The core intentionally exposes immutable accessors for the managed build, context, input, factory, and runner. It does not expose mutable references that could bypass generation changes and baseline invalidation.

### Current output access

`current_output` returns `None` until the current build has been calculated successfully. After baseline creation, it returns a reference to the stored finalized output.

```rust
if let Some(output) = core.current_output() {
    render_current_output(output);
}
```

---

## 12. Candidate Build Construction and Comparison

### `BuildCandidateFactory`

`BuildCandidateFactory<B>` creates an owned hypothetical build from the current build and a candidate description.

```rust
pub trait BuildCandidateFactory<B> {
    type Candidate;
    type Error;

    fn create_candidate(
        &self,
        current: &B,
        candidate: &Self::Candidate,
    ) -> Result<B, Self::Error>;
}
```

The candidate description can represent an item replacement, a passive-node addition, a configuration change, or any other application-specific modification. The trait does not require the build type itself to implement `Clone`; the concrete factory decides how the candidate is constructed.

### Direct candidate comparison

`compare_candidate_build` accepts an already constructed candidate build. It ensures that a current baseline exists, calculates the candidate using the same context, factory, and input, clones the cached current output, and compares the two outputs.

```rust
let comparison =
    core.compare_candidate_build(&candidate_build)?;
```

A failure while creating the current baseline is reported as `BuildCandidateComparisonError::Current`. A candidate calculation failure is reported as `BuildCandidateComparisonError::Candidate`.

The candidate result never replaces the current baseline.

### Candidate-description comparison

`compare_candidate_with` first asks a `BuildCandidateFactory` to construct the candidate build, then delegates to `compare_candidate_build`.

```rust
let comparison = core.compare_candidate_with(
    &candidate_factory,
    &candidate_description,
)?;
```

Candidate construction happens before baseline creation. If candidate construction fails, the calculation pipeline is not started and the current baseline is not created or modified.

---

## 13. Output Comparison

### `CalculationOutputComparator`

`CalculationOutputComparator<O>` defines how two finalized outputs are compared.

```rust
pub trait CalculationOutputComparator<O> {
    type Difference;

    fn compare(
        &self,
        baseline: &O,
        candidate: &O,
    ) -> Self::Difference;
}
```

The comparator is infallible in the current API. It receives borrowed outputs and produces an owned difference value.

### `CalculationComparison`

`CalculationComparison<O, D>` owns the baseline output, candidate output, and calculated difference.

```rust
let comparison = CalculationComparison::between(
    baseline,
    candidate,
    &comparator,
);
```

Because the comparison owns its data, a returned comparison does not borrow `BuildCalculationCore`.

### `CalculationComparisonRunner`

`CalculationComparisonRunner<C>` owns a comparator and supports both low-level and high-level comparison workflows.

`compare_outputs` compares two already finalized outputs. `compare_from_input` calculates a baseline and candidate from active effect collections and then compares the results. `calculate_baseline_from_input` creates a revision-tagged baseline. `compare_candidate_from_input` verifies that the stored baseline revision still matches the caller-provided current revision before calculating the candidate.

The revision-aware API reports `CandidateComparisonError::StaleBaseline` before candidate calculation when revisions do not match.

### Numeric and value differences

`NumericStatDifference` stores baseline, candidate, absolute difference, and an optional relative percentage. Relative percentage is `None` when the baseline is zero.

`StatValueDifference<T>` stores baseline and candidate values without assuming numeric subtraction. When `T: PartialEq`, `is_changed` reports whether the values differ.

---

## 14. Error Boundaries and Failure Semantics

The library preserves stage-specific errors instead of flattening all failures into one opaque type.

Item editing separates rule validation failures from missing-modifier failures for remove and replace operations. Item validation returns `ItemValidationFailure`, which preserves the rejected item. Item effect collection distinguishes definition-provider failures from modifier-resolution failures.

Effect calculation distinguishes planning, application, finalization, and optional accumulator-creation failures. Build calculation further separates build collection, condition evaluation, and calculation failures. Candidate workflows distinguish current-baseline failures, candidate failures, and candidate-construction failures.

The current implementations also preserve several important commit boundaries. Item-editor validation occurs before mutation. Item effect collection builds a local result before extending the destination collection. Multi-item collection uses a temporary collection. Planning completes before effects are applied. A current calculation inserts a baseline only after a finalized output has been produced. Candidate calculation never replaces the baseline.

These boundaries make failures easier to diagnose and prevent partially committed high-level state.

---

## 15. Ownership, Lifetimes, and Dispatch

The item model and raw effect collection own their data. `ActiveEffectCollection<'a, G>` borrows sourced entries from an `EffectCollection<G>`. `EffectExecutionPlan<'a, G>` and `EffectSelectionRejection<'a, G>` continue borrowing those same entries. `EffectCalculationOutput<'a, G, O>` therefore borrows the source effect collection through its execution plan while owning the finalized output.

The accumulator is owned by one calculation operation. `EffectAccumulatorFactory` creates it, `EffectApplier` mutates it through `&mut`, and `EffectAccumulatorFinalizer` consumes it.

The calculation pipeline uses generic static dispatch. Types such as the collector, evaluator, planner, applier, finalizer, factory, and comparator are generic parameters. Rust knows their concrete implementations during compilation and monomorphizes the pipeline for those types.

This model preserves precise associated-type relationships and allows the compiler to optimize direct calls. It also means that replacing one implementation with another changes the concrete type of the containing runner or core.

---

## 16. Required and Optional Integrations

A project that only needs item storage can implement `Game` and use `ItemInstance` directly. Safe editing additionally requires an `ItemRule` and `ItemEditor`. Validation requires an `ItemValidator`, and many implementations will also use a `ModifierValidator` and `ModifierDefinitionProvider`.

Item-to-effect conversion requires a modifier-definition provider and a `ModifierEffectResolver`. Direct non-item sources use `EffectSource`. Conditional effects require an `EffectConditionEvaluator`.

A complete calculation pipeline requires an `EffectPlanner`, `EffectApplier`, `EffectAccumulatorFactory`, and `EffectAccumulatorFinalizer`. A project using the generic planning algorithm implements `EffectPlanningPolicy` and constructs `EffectExecutionPlanner`.

Build-level calculation requires `BuildEffectCollector`. Stateful current-build caching requires `BuildCalculationCore`. Final output comparison requires `CalculationOutputComparator`. Candidate descriptions require `BuildCandidateFactory`.

`PassiveNodeProvider` is optional and is only needed when the concrete project has a passive-node lookup layer. Text parsers are optional when item data is constructed or deserialized through another interface. The stateful core is optional when the application only needs one-shot calculations.

---

## 17. Step-by-Step Usage Guide

This section builds one complete example gradually. Every step introduces only the types and implementations required by the next stage. 

All snippets in this section belong to the same Rust module.

### Step 1: Import the library contracts

The example uses the item, effect, planning, build-calculation, and comparison APIs. `Arc` is used later so a candidate build can reuse unchanged validated items without requiring `ItemInstance` itself to implement `Clone`.

```rust
use std::{
    convert::Infallible,
    sync::Arc,
};

use exile_core::{
    effect::{
        BuildCalculationCore,
        BuildCalculationRunner,
        BuildCandidateFactory,
        BuildEffectCollector,
        CalculationComparisonRunner,
        CalculationOutputComparator,
        EffectAccumulatorFactory,
        EffectAccumulatorFinalizer,
        EffectApplier,
        EffectCollection,
        EffectCollectionEvaluator,
        EffectConditionEvaluator,
        EffectEntry,
        EffectExecutionPlanner,
        EffectPlanningPolicy,
        ItemEffectCollectionError,
        ItemEffectCollector,
        ModifierEffectResolver,
        NumericStatDifference,
    },
    game::{
        Game,
        ModifierDefinitionIdentity,
    },
    item::{
        ItemEditor,
        ItemInstance,
        ItemRule,
        ItemValidator,
        ModifierDefinitionProvider,
        ModifierInstanceId,
        ModifierValidator,
        Unvalidated,
        Validated,
    },
};
```

### Step 2: Define the game-specific domain types

The example models one item base, one modifier definition, one rolled modifier value, one effect, one condition, and one source identifier. These types form the vocabulary connected by `Game`.

```rust
pub struct MyGame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyItemBase {
    Boots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyItemState {
    pub item_level: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MyModifierDefinitionId {
    MovementSpeed,
}

#[derive(Debug)]
pub struct MyModifierDefinition {
    id: MyModifierDefinitionId,
    required_item_level: u16,
    minimum_roll: u16,
    maximum_roll: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyModifierInstance {
    pub value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEffect {
    IncreasedMovementSpeed {
        percent: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEffectCondition {
    EnemyOnFullLife,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MyEffectSourceId {
    PassiveNode(u32),
}
```

`MyModifierDefinition` must expose its identifier because `Game::ModifierDefinition` is constrained by `ModifierDefinitionIdentity`.

```rust
impl ModifierDefinitionIdentity for MyModifierDefinition {
    type Id = MyModifierDefinitionId;

    fn modifier_definition_id(&self) -> Self::Id {
        self.id
    }
}
```

The `Game` implementation now connects every generic associated type to the concrete types declared above.

```rust
impl Game for MyGame {
    type ItemBase = MyItemBase;
    type ItemState = MyItemState;

    type ModifierDefinitionId = MyModifierDefinitionId;
    type ModifierDefinition = MyModifierDefinition;
    type ModifierInstance = MyModifierInstance;

    type Effect = MyEffect;
    type EffectCondition = MyEffectCondition;
    type EffectSourceId = MyEffectSourceId;
}
```

At this point the crate knows which types belong to `MyGame`, but it still does not know where modifier definitions come from or how they are validated.

### Step 3: Create the modifier-definition provider

`MyDefinitionProvider` owns the movement-speed definition. The value that later appears as `movement_speed_definition` will be borrowed from this provider rather than appearing as an variable.

```rust
pub struct MyDefinitionProvider {
    movement_speed: MyModifierDefinition,
}

impl MyDefinitionProvider {
    pub fn new() -> Self {
        Self {
            movement_speed: MyModifierDefinition {
                id: MyModifierDefinitionId::MovementSpeed,
                required_item_level: 1,
                minimum_roll: 1,
                maximum_roll: 40,
            },
        }
    }

    pub fn movement_speed_definition(
        &self,
    ) -> &MyModifierDefinition {
        &self.movement_speed
    }
}

impl Default for MyDefinitionProvider {
    fn default() -> Self {
        Self::new()
    }
}
```

The provider implements the lookup contract used by item validation and item effect collection. This example has only one known identifier, so lookup is infallible.

```rust
impl ModifierDefinitionProvider<MyGame>
    for MyDefinitionProvider
{
    type Error = Infallible;

    fn definition(
        &self,
        id: &MyModifierDefinitionId,
    ) -> Result<&MyModifierDefinition, Self::Error> {
        match id {
            MyModifierDefinitionId::MovementSpeed => {
                Ok(&self.movement_speed)
            }
        }
    }
}
```

The definition provider is now fully constructed and can supply the exact definition used by editing, validation, and effect resolution.

### Step 4: Implement modifier validation

The modifier validator checks both item-level requirements and roll bounds. Its error describes which constraint failed.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MyModifierValidationError {
    ItemLevelTooLow {
        required: u16,
        actual: u16,
    },
    RollOutsideAllowedRange {
        minimum: u16,
        maximum: u16,
        actual: u16,
    },
}

pub struct MyModifierValidator;

impl ModifierValidator<MyGame> for MyModifierValidator {
    type Error = MyModifierValidationError;

    fn validate_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        let item_level = item.state().item_level;

        if item_level < definition.required_item_level {
            return Err(
                MyModifierValidationError::ItemLevelTooLow {
                    required: definition.required_item_level,
                    actual: item_level,
                },
            );
        }

        if modifier.value < definition.minimum_roll
            || modifier.value > definition.maximum_roll
        {
            return Err(
                MyModifierValidationError::RollOutsideAllowedRange {
                    minimum: definition.minimum_roll,
                    maximum: definition.maximum_roll,
                    actual: modifier.value,
                },
            );
        }

        Ok(())
    }
}
```

The validator is independent from the editor. It only inspects an item, definition, and modifier instance.

### Step 5: Implement item editing rules

`ItemEditor` delegates every requested mutation to an `ItemRule`. This example reuses `MyModifierValidator` for add and replace operations. State replacement and modifier removal are always permitted.

```rust
pub struct MyItemRule;

impl ItemRule<MyGame> for MyItemRule {
    type Error = MyModifierValidationError;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        MyModifierValidator.validate_modifier(
            item,
            definition,
            modifier,
        )
    }

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        _target_id: ModifierInstanceId,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        MyModifierValidator.validate_modifier(
            item,
            definition,
            modifier,
        )
    }

    fn validate_replace_state(
        &self,
        _item: &ItemInstance<MyGame, Unvalidated>,
        _new_state: &MyItemState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_remove_modifier(
        &self,
        _item: &ItemInstance<MyGame, Unvalidated>,
        _id: ModifierInstanceId,
        _modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
```

The editor can now safely add the movement-speed modifier because both the definition and the mutation rule are available.

### Step 6: Implement complete item validation

`MyItemValidator` borrows the definition provider and owns a modifier validator. Complete validation walks over every stored modifier, resolves its definition, and validates the stored instance in item context.

```rust
pub struct MyItemValidator<'a> {
    definitions: &'a MyDefinitionProvider,
    modifier_validator: MyModifierValidator,
}

impl<'a> MyItemValidator<'a> {
    pub fn new(
        definitions: &'a MyDefinitionProvider,
    ) -> Self {
        Self {
            definitions,
            modifier_validator: MyModifierValidator,
        }
    }
}

impl ItemValidator<MyGame> for MyItemValidator<'_> {
    type Error = MyModifierValidationError;

    fn validate_item(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
    ) -> Result<(), Self::Error> {
        for stored_modifier in item.modifiers() {
            let definition = self
                .definitions
                .definition(stored_modifier.definition_id())
                .expect(
                    "MyDefinitionProvider uses an infallible lookup",
                );

            self.modifier_validator.validate_modifier(
                item,
                definition,
                stored_modifier.modifier(),
            )?;
        }

        Ok(())
    }
}
```

The `item_validator` used later will be created with `MyItemValidator::new(&definition_provider)`. Its origin is therefore explicit.

### Step 7: Create and validate boots

The helper below brings the previous item components together. It obtains `movement_speed_definition` from the provider, creates an unvalidated item, edits it through `ItemEditor`, creates the item validator, and finally returns a validated item.

```rust
fn create_validated_boots(
    definition_provider: &MyDefinitionProvider,
    movement_speed_percent: u16,
) -> ItemInstance<MyGame, Validated> {
    let movement_speed_definition =
        definition_provider
            .movement_speed_definition();

    let mut item = ItemInstance::<MyGame>::new(
        MyItemBase::Boots,
        MyItemState {
            item_level: 86,
        },
    );

    let editor = ItemEditor::new(MyItemRule);

    editor
        .add_modifier(
            &mut item,
            movement_speed_definition,
            MyModifierInstance {
                value: movement_speed_percent,
            },
        )
        .expect(
            "the movement-speed modifier must be valid",
        );

    let item_validator =
        MyItemValidator::new(
            definition_provider,
        );

    item.validate(&item_validator)
        .expect(
            "the boots must pass complete item validation",
        )
}
```

No value in this helper is implicit. The definition comes from the provider, the editor owns `MyItemRule`, and the validator is constructed before `validate` is called.

### Step 8: Resolve modifier instances into effects

Validated items cannot enter the calculation pipeline until their stored modifiers are translated into `EffectEntry` values. `MyModifierEffectResolver` performs that conversion.

```rust
pub struct MyModifierEffectResolver;

impl ModifierEffectResolver<MyGame>
    for MyModifierEffectResolver
{
    type Error = Infallible;

    fn resolve_modifier_effects(
        &self,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<Vec<EffectEntry<MyGame>>, Self::Error> {
        match definition.id {
            MyModifierDefinitionId::MovementSpeed => {
                Ok(vec![
                    EffectEntry::unconditional(
                        MyEffect::IncreasedMovementSpeed {
                            percent: modifier.value,
                        },
                    ),
                ])
            }
        }
    }
}
```

The resolver produces a raw effect without provenance. `ItemEffectCollector` will later attach `EffectOrigin::ItemModifier`.

### Step 9: Implement condition evaluation

The example effect is unconditional, but the game still needs to provide the associated condition type required by `Game`. The evaluator below shows how the declared condition would be resolved if a conditional effect were added later.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyConditionContext {
    pub enemy_on_full_life: bool,
}

pub struct MyConditionEvaluator;

impl EffectConditionEvaluator<MyGame>
    for MyConditionEvaluator
{
    type Context = MyConditionContext;
    type Error = Infallible;

    fn evaluate_condition(
        &self,
        condition: &MyEffectCondition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        match condition {
            MyEffectCondition::EnemyOnFullLife => {
                Ok(context.enemy_on_full_life)
            }
        }
    }
}
```

### Step 10: Implement effect planning

The example has only one effect category, so every effect receives the same phase and priority. There are no exclusive conflicts and no winner-selection groups.

```rust
pub struct MyPlanningPolicy;

impl EffectPlanningPolicy<MyGame>
    for MyPlanningPolicy
{
    type Phase = u8;
    type Priority = u8;
    type ConflictKey = ();
    type SelectionKey = ();

    fn phase(
        &self,
        _effect: &MyEffect,
    ) -> Self::Phase {
        0
    }

    fn priority(
        &self,
        _effect: &MyEffect,
    ) -> Self::Priority {
        0
    }

    fn conflict_key(
        &self,
        _effect: &MyEffect,
    ) -> Option<Self::ConflictKey> {
        None
    }

    fn selection_key(
        &self,
        _effect: &MyEffect,
    ) -> Option<Self::SelectionKey> {
        None
    }

    fn prefers(
        &self,
        _candidate: &MyEffect,
        _current: &MyEffect,
    ) -> bool {
        false
    }
}
```

`EffectExecutionPlanner<MyPlanningPolicy>` can now produce a deterministic execution plan for `MyGame`.

### Step 11: Define calculation input, accumulator, and output

The calculation begins with base movement speed, mutates an accumulator, and produces immutable final stats.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyCalculationInput {
    pub base_movement_speed_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyAccumulator {
    movement_speed_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyFinalStats {
    pub movement_speed_percent: u32,
}
```

`MyCalculationInput` is the value passed to the accumulator factory. `MyAccumulator` is internal mutable calculation state. `MyFinalStats` is the finalized output used by the comparison layer.

### Step 12: Implement the accumulator factory

The factory creates a fresh accumulator for every calculation.

```rust
pub struct MyAccumulatorFactory;

impl EffectAccumulatorFactory
    for MyAccumulatorFactory
{
    type Input = MyCalculationInput;
    type Accumulator = MyAccumulator;
    type Error = Infallible;

    fn create(
        &self,
        input: &Self::Input,
    ) -> Result<Self::Accumulator, Self::Error> {
        Ok(MyAccumulator {
            movement_speed_percent:
                input.base_movement_speed_percent,
        })
    }
}
```

### Step 13: Implement effect application

`MyEffectApplier` performs the game-specific arithmetic. It uses checked addition and commits the accumulator update only after the arithmetic succeeds.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyApplyError {
    MovementSpeedOverflow,
}

pub struct MyEffectApplier;

impl EffectApplier<MyGame> for MyEffectApplier {
    type Accumulator = MyAccumulator;
    type Error = MyApplyError;

    fn apply_effect(
        &self,
        effect: &MyEffect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            MyEffect::IncreasedMovementSpeed {
                percent,
            } => {
                let updated = accumulator
                    .movement_speed_percent
                    .checked_add(u32::from(*percent))
                    .ok_or(
                        MyApplyError::MovementSpeedOverflow,
                    )?;

                accumulator.movement_speed_percent = updated;

                Ok(())
            }
        }
    }
}
```

### Step 14: Implement finalization

The finalizer consumes the accumulator and returns the immutable output.

```rust
pub struct MyFinalizer;

impl EffectAccumulatorFinalizer for MyFinalizer {
    type Accumulator = MyAccumulator;
    type Output = MyFinalStats;
    type Error = Infallible;

    fn finalize(
        &self,
        accumulator: Self::Accumulator,
    ) -> Result<Self::Output, Self::Error> {
        Ok(MyFinalStats {
            movement_speed_percent:
                accumulator.movement_speed_percent,
        })
    }
}
```

The low-level calculation pipeline is now complete: a factory creates `MyAccumulator`, `MyEffectApplier` mutates it, and `MyFinalizer` produces `MyFinalStats`.

### Step 15: Define the build model

The build contains one pair of validated boots. `Arc` allows the current and candidate build containers to share unchanged item values without requiring `ItemInstance` to implement `Clone`.

```rust
#[derive(Clone)]
pub struct MyBuild {
    pub boots: Arc<
        ItemInstance<MyGame, Validated>,
    >,
}
```

The build is deliberately game-specific. `exile-core` only requires a `BuildEffectCollector` implementation that can turn this structure into an `EffectCollection`.

### Step 16: Implement build effect collection

`MyBuildEffectCollector` owns an `ItemEffectCollector` that borrows the definition provider and modifier resolver created in earlier steps.

```rust
pub struct MyBuildEffectCollector<'a> {
    item_collector: ItemEffectCollector<
        'a,
        MyDefinitionProvider,
        MyModifierEffectResolver,
    >,
}

impl<'a> MyBuildEffectCollector<'a> {
    pub fn new(
        definition_provider: &'a MyDefinitionProvider,
        resolver: &'a MyModifierEffectResolver,
    ) -> Self {
        Self {
            item_collector:
                ItemEffectCollector::new(
                    definition_provider,
                    resolver,
                ),
        }
    }
}
```

The trait implementation creates a fresh effect collection and collects effects from the validated boots stored in the build.

```rust
impl BuildEffectCollector<MyGame>
    for MyBuildEffectCollector<'_>
{
    type Build = MyBuild;
    type Error = ItemEffectCollectionError<
        Infallible,
        Infallible,
    >;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<MyGame>, Self::Error> {
        let mut effects =
            EffectCollection::<MyGame>::new();

        effects.collect_from_item(
            &self.item_collector,
            build.boots.as_ref(),
        )?;

        Ok(effects)
    }
}
```

At this point a complete `MyBuild` can be transformed into sourced effects.

### Step 17: Construct the build calculation runner

The following aliases keep the concrete planner and runner types readable.

```rust
type MyPlanner =
    EffectExecutionPlanner<MyPlanningPolicy>;

type MyRunner<'a> = BuildCalculationRunner<
    MyBuildEffectCollector<'a>,
    MyConditionEvaluator,
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
>;
```

The helper constructs every runner dependency in the same order expected by `BuildCalculationRunner::new`.

```rust
fn create_runner<'a>(
    definition_provider: &'a MyDefinitionProvider,
    modifier_resolver: &'a MyModifierEffectResolver,
) -> MyRunner<'a> {
    let build_collector =
        MyBuildEffectCollector::new(
            definition_provider,
            modifier_resolver,
        );

    let evaluator =
        EffectCollectionEvaluator::new(
            MyConditionEvaluator,
        );

    let planner =
        EffectExecutionPlanner::new(
            MyPlanningPolicy,
        );

    let calculator =
        exile_core::effect::EffectCalculator::new(
            MyEffectApplier,
            MyFinalizer,
            planner,
        );

    BuildCalculationRunner::new(
        build_collector,
        evaluator,
        calculator,
    )
}
```

Nothing is hidden behind the name `runner`. The helper shows the collector, evaluator, planner, calculator, applier, and finalizer used to construct it.

### Step 18: Define output comparison

The comparator describes how two `MyFinalStats` values become one owned difference.

```rust
pub struct MyOutputComparator;

impl CalculationOutputComparator<MyFinalStats>
    for MyOutputComparator
{
    type Difference = NumericStatDifference;

    fn compare(
        &self,
        baseline: &MyFinalStats,
        candidate: &MyFinalStats,
    ) -> Self::Difference {
        NumericStatDifference::between(
            f64::from(
                baseline.movement_speed_percent,
            ),
            f64::from(
                candidate.movement_speed_percent,
            ),
        )
    }
}
```

The comparator is now available for `CalculationComparisonRunner`.

### Step 19: Define candidate construction

The candidate description contains replacement boots. `MyBuildCandidateFactory` clones the current build container, replaces the boots in the clone, and returns a new owned candidate build.

```rust
pub struct ReplaceBootsCandidate {
    pub boots: Arc<
        ItemInstance<MyGame, Validated>,
    >,
}

pub struct MyBuildCandidateFactory;

impl BuildCandidateFactory<MyBuild>
    for MyBuildCandidateFactory
{
    type Candidate = ReplaceBootsCandidate;
    type Error = Infallible;

    fn create_candidate(
        &self,
        current: &MyBuild,
        candidate: &Self::Candidate,
    ) -> Result<MyBuild, Self::Error> {
        let mut candidate_build = current.clone();

        candidate_build.boots =
            Arc::clone(&candidate.boots);

        Ok(candidate_build)
    }
}
```

The current build remains unchanged because the factory returns a separate `MyBuild`.

### Step 20: Define the managed core type

The complete concrete `BuildCalculationCore` type is long, so a semantic alias makes the final example easier to read.

```rust
type MyCore<'a> = BuildCalculationCore<
    MyGame,
    MyBuildEffectCollector<'a>,
    MyConditionEvaluator,
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
    MyAccumulatorFactory,
    MyOutputComparator,
>;
```

All components referenced by this alias have already been declared in previous steps.

### Step 21: Calculate the current build and compare a candidate

The final function now constructs every runtime value before using it. It creates the definition provider and resolver, validates current and candidate boots, constructs the current build, creates the runner and comparison runner, defines the condition context and calculation input, creates the managed core, calculates the baseline, and compares the replacement-boots candidate.

```rust
fn main() {
    let definition_provider =
        MyDefinitionProvider::new();

    let modifier_resolver =
        MyModifierEffectResolver;

    let current_boots = Arc::new(
        create_validated_boots(
            &definition_provider,
            25,
        ),
    );

    let candidate_boots = Arc::new(
        create_validated_boots(
            &definition_provider,
            35,
        ),
    );

    let current_build = MyBuild {
        boots: Arc::clone(&current_boots),
    };

    let runner = create_runner(
        &definition_provider,
        &modifier_resolver,
    );

    let comparison_runner =
        CalculationComparisonRunner::new(
            MyOutputComparator,
        );

    let context = MyConditionContext {
        enemy_on_full_life: false,
    };

    let calculation_input = MyCalculationInput {
        base_movement_speed_percent: 100,
    };

    let mut core: MyCore<'_> =
        BuildCalculationCore::new(
            current_build,
            context,
            calculation_input,
            MyAccumulatorFactory,
            runner,
            comparison_runner,
        );

    let current_output = core
        .calculate_current()
        .expect(
            "the current build must calculate successfully",
        );

    assert_eq!(
        current_output.movement_speed_percent,
        125,
    );

    let candidate_description =
        ReplaceBootsCandidate {
            boots: candidate_boots,
        };

    let comparison = core
        .compare_candidate_with(
            &MyBuildCandidateFactory,
            &candidate_description,
        )
        .expect(
            "the candidate build must calculate successfully",
        );

    assert_eq!(
        comparison
            .baseline()
            .movement_speed_percent,
        125,
    );

    assert_eq!(
        comparison
            .candidate()
            .movement_speed_percent,
        135,
    );

    assert_eq!(
        comparison
            .difference()
            .absolute(),
        10.0,
    );
}
```

The current calculation begins with `100%` base movement speed and applies the `25%` modifier from the current boots, producing a cached baseline of `125%`. The candidate factory creates a new build containing boots with a `35%` modifier. Candidate calculation therefore produces `135%`, and `NumericStatDifference` reports an absolute improvement of `10`.
---

## 18. Module Organization and Public API

The crate root exposes the `effect`, `game`, and `item` modules.

```rust
pub mod effect;
pub mod game;
pub mod item;
```

The item module is divided into definition, editing, model, parsing, and validation submodules. Each submodule keeps its leaf modules private and re-exports the intended public types through its own `mod.rs`. The main item module then re-exports the public surface from those submodules.

The effect module follows the same facade pattern. Model, source, evaluation, and calculation are organized internally, while consumers import the public types through `exile_core::effect`.

This structure keeps physical file organization separate from the external API.

```rust
use exile_core::{
    effect::{
        BuildCalculationCore,
        BuildCalculationRunner,
        EffectCalculator,
        EffectCollection,
        EffectExecutionPlanner,
    },
    game::Game,
    item::{
        ItemEditor,
        ItemInstance,
        ItemValidator,
        Validated,
    },
};
```

---
