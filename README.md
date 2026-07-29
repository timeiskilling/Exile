# `exile-core`: Architecture, Semantics, and Usage Guide

## Table of Contents

1. [High-Level Architecture & Core Flow](#section-1)
2. [The `Game`](#section-2)
3. [Item Model, Editing, Parsing, and Validation](#section-3)
4. [Effect Model and Provenance](#section-4)
5. [Effect Sources and Item Effect Resolution](#section-5)
6. [Conditional Effect Evaluation](#section-6)
7. [Planning and Execution Order](#section-7)
8. [Accumulator Creation, Effect Application, and Finalization](#section-8)
9. [`EffectCalculator` and Detailed Calculation Output](#section-9)
10. [Build-Level Calculation](#section-10)
11. [Stateful Calculation with `BuildCalculationCore`](#section-11)
12. [Candidate Build Construction and Comparison](#section-12)
13. [Output Comparison](#section-13)
14. [Error Boundaries and Failure Semantics](#section-14)
15. [Ownership, Lifetimes, and Dispatch](#section-15)
16. [Required and Optional Integrations](#section-16)
17. [Step-by-Step Usage Guide](#section-17)
18. [Module Organization and Public API](#section-18)
19. [Known Design Questions and Open Issues](#section-19)

---

<a id="section-1"></a>

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


<a id="section-2"></a>

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

<a id="section-3"></a>

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

<a id="section-4"></a>

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

<a id="section-5"></a>

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

<a id="section-6"></a>

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

<a id="section-7"></a>

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

`EffectPlanningPolicy` methods are expected to behave deterministically for the same effect and policy state. The generic planner computes `selection_key` once for each active entry, stores the result for the duration of planning, and reuses that stored key during both winner selection and rejection construction. This avoids repeated game-specific classification and guarantees that both passes observe the same selection group for an entry.

### `EffectExecutionPlanner`

`EffectExecutionPlanner<P>` performs planning in three stages. It first builds an ordered preliminary plan using phase, priority, and original collection position. It then validates exclusive conflict keys. Finally, it selects one winner for every selection group and records the rejected effects.

The original collection index is used as the final ordering key. Effects with the same phase and priority therefore remain deterministic.

When two effects share the same conflict key, planning fails with `EffectExecutionPlanValidationError`. The error owns the conflict key and cloned origins of the first and second conflicting effects.

Winner selection is performed in two passes. The planner first determines the final winner index for each selection key. It then constructs the selected entry list and creates `EffectSelectionRejection` records for every losing entry. Every rejection points to the final winner rather than an intermediate candidate.

### `EffectExecutionPlan`

`EffectExecutionPlan<'a, G>` contains ordered selected entries and selection-rejection metadata. It provides iteration over sourced entries and effects, as well as access to rejection records and their count.

Plan construction is internal to the crate. External callers obtain a plan through `EffectPlanner::plan`, which preserves the complete planning contract: deterministic ordering, conflict validation, winner selection, and rejection recording. The constructors used to assemble a plan directly are crate-private so an arbitrary ordered list cannot be presented as a fully validated execution plan.

---

<a id="section-8"></a>

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

An individual `apply_effect` implementation should commit atomically when one effect requires several checked operations. Temporary values should be computed first, and accumulator fields should be updated only after all operations for that effect succeed.

### `EffectCollectionApplier`

`EffectCollectionApplier<A>` applies every effect in an `EffectExecutionPlan`. It trusts the plan, preserves its order, and stops on the first application error.

`apply_all` is the low-level in-place path. It receives `&mut Accumulator`, so the caller retains ownership and can observe mutations committed before a later effect fails. The method does not provide rollback.

```rust
collection_applier.apply_all(
    &plan,
    &mut accumulator,
)?;
```

`apply_all_owned` receives the accumulator by value and returns it only after every effect succeeds.

```rust
let accumulator = collection_applier
    .apply_all_owned(
        &plan,
        accumulator,
    )?;
```

On failure, the owned accumulator is dropped inside the method. Earlier mutations are not reversed, but the partially applied value is not returned. This is an ownership boundary rather than a transaction boundary.

The high-level calculator uses this owned path. Its complete ownership and error behavior is defined in [Section 9](#section-9). The unresolved API trade-off is tracked in [Section 19](#section-19).

If an accumulator contains `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, or another shared mutable handle, dropping the accumulator wrapper does not undo mutations visible through other owners. Integrations that require isolation should avoid externally shared mutable accumulator state or provide their own transaction strategy.

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

<a id="section-9"></a>

## 9. `EffectCalculator` and Detailed Calculation Output

`EffectCalculator<A, F, P>` combines an `EffectCollectionApplier<A>`, a finalizer, and a planner.

```rust
let calculator = EffectCalculator::new(
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
);
```

The calculator exposes four calculation paths. `calculate` accepts an active effect collection and an already created accumulator, then returns only the finalized output. `calculate_detailed` performs the same pipeline but returns `EffectCalculationOutput<'a, G, O>`, which owns the finalized output together with the execution plan that produced it.

`calculate_from_input` and `calculate_from_input_detailed` first ask an `EffectAccumulatorFactory` to create the accumulator, then delegate to the same calculation pipeline.

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

The non-detailed methods are thin wrappers over the detailed methods. They consume `EffectCalculationOutput` with `into_output`, return the finalized output, and discard the execution plan. Planning, application, and finalization therefore have one canonical implementation.

### Accumulator ownership

`calculate` and `calculate_detailed` take the accumulator by value. Passing a non-`Copy` accumulator transfers ownership permanently into that calculation call.

A planning error drops the accumulator before application. An application error follows the owned behavior described in [Section 8](#section-8): the accumulator may contain earlier successful mutations, but it is dropped rather than returned. A finalization error occurs after ownership has moved into the finalizer.

This contract provides consumption, not rollback. A failed calculation returns only its stage-specific error and never returns the accumulator. The possible introduction of a recoverable alternative is discussed in [Section 19](#section-19).

### Calculation errors

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

The conversion from `EffectCalculationError` to `EffectCalculationFromInputError` preserves the `Plan`, `Apply`, and `Finalize` variants. `CreateAccumulator` can only originate before the owned calculation pipeline begins.

### Detailed output lifetime

Because detailed output owns an execution plan that contains references to sourced effects, `EffectCalculationOutput<'a, G, O>` remains tied to the lifetime of the source `EffectCollection`. Calling `into_output` discards the plan and returns the owned finalized output without that plan lifetime.

`EffectCalculationOutput::new` is crate-private. External code can inspect or consume a detailed result, but it cannot manually combine an unrelated output and execution plan into a value that appears to represent one calculation.

---

<a id="section-10"></a>

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

<a id="section-11"></a>

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

<a id="section-12"></a>

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

<a id="section-13"></a>

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

`NumericStatDifference` stores the baseline, candidate, absolute difference, and an optional relative percentage. The absolute difference is calculated as `candidate - baseline`. A zero absolute result is normalized to positive `0.0`, so the stored value does not retain negative zero.

Relative percentage is available only when the baseline, candidate, and absolute difference are finite and the baseline is not zero. A zero baseline, `NaN`, or either infinity produces `None` rather than `Some(NaN)` or an infinite percentage.

`is_changed` compares the source values directly with `baseline != candidate`. Its behavior follows Rust `f64` and IEEE 754 comparison semantics.

| Baseline | Candidate | `is_changed()` |
|---:|---:|:---:|
| `100.0` | `100.0` | `false` |
| `100.0` | `125.0` | `true` |
| `0.0` | `-0.0` | `false` |
| `+∞` | `+∞` | `false` |
| `NaN` | `NaN` | `true` |

Positive and negative zero compare as equal. Equal positive infinities also compare as equal. `NaN` compares unequal to every value, including another `NaN`, so two `NaN` inputs are reported as changed. `is_positive` and `is_negative` inspect the absolute difference; both return `false` when that difference is `NaN`.

The relative formula remains `absolute / baseline * 100.0`. For a negative finite baseline, the sign therefore follows the mathematical denominator rather than using `abs(baseline)`. A game that requires different domain semantics should implement its own difference type or comparator.

`StatValueDifference<T>` stores baseline and candidate values without assuming numeric subtraction. When `T: PartialEq`, `is_changed` reports whether the values differ.

---

<a id="section-14"></a>

## 14. Error Boundaries and Failure Semantics

The library preserves stage-specific errors instead of flattening all failures into one opaque type.

Item editing separates rule validation failures from missing-modifier failures for remove and replace operations. Revision capacity is checked before a mutation is committed, so revision overflow does not leave an item changed without the corresponding revision update. Item validation returns `ItemValidationFailure`, which preserves the rejected item. Item effect collection distinguishes definition-provider failures from modifier-resolution failures.

Effect calculation distinguishes planning, application, finalization, and optional accumulator-creation failures. Build calculation further separates build collection, condition evaluation, and calculation failures. Candidate workflows distinguish current-baseline failures, candidate failures, and candidate-construction failures.

Several operations use explicit commit boundaries. Item-editor validation occurs before mutation. Item effect collection builds a local result before extending the destination collection. Multi-item collection uses a temporary collection. Planning completes before application begins. A current calculation inserts a baseline only after a finalized output has been produced. Candidate calculation never replaces the baseline.

Accumulator failure semantics are documented once at their corresponding abstraction levels. The low-level difference between borrowed and owned application is described in [Section 8](#section-8), while high-level calculator ownership is described in [Section 9](#section-9).

These boundaries protect committed high-level state and make failures easier to diagnose. They do not imply that every low-level operation is transactional.

---

<a id="section-15"></a>

## 15. Ownership, Lifetimes, and Dispatch

The item model and raw effect collection own their data. `ActiveEffectCollection<'a, G>` borrows sourced entries from an `EffectCollection<G>`. `EffectExecutionPlan<'a, G>` and `EffectSelectionRejection<'a, G>` continue borrowing those same entries. `EffectCalculationOutput<'a, G, O>` therefore owns the finalized output and the plan container while still borrowing the original sourced entries through that plan.

Accumulator ownership follows the contract in [Section 9](#section-9). Resource-management consequences for shared mutable accumulator state are covered in [Section 8](#section-8), so they are not repeated here.

The effect model uses manual trait implementations where a derive would impose unnecessary bounds on the game marker type. For example, `EffectEntry<G>: Debug` depends on `G::Effect: Debug` and `G::EffectCondition: Debug`, not on `G: Debug`. `EffectOrigin<G>` follows the same principle for definition and source identifiers. A concrete `Game` marker does not need unrelated derives merely to use these wrappers.

The calculation pipeline uses generic static dispatch. Types such as the collector, evaluator, planner, applier, finalizer, factory, and comparator are generic parameters. Rust knows their concrete implementations during compilation and monomorphizes the pipeline for those types.

This model preserves precise associated-type relationships and allows the compiler to optimize direct calls. It also means that replacing one implementation with another changes the concrete type of the containing runner or core.

---

<a id="section-16"></a>

## 16. Required and Optional Integrations

A project that only needs item storage can implement `Game` and use `ItemInstance` directly. Safe editing additionally requires an `ItemRule` and `ItemEditor`. Validation requires an `ItemValidator`, and many implementations will also use a `ModifierValidator` and `ModifierDefinitionProvider`.

Item-to-effect conversion requires a modifier-definition provider and a `ModifierEffectResolver`. Direct non-item sources use `EffectSource`. Conditional effects require an `EffectConditionEvaluator`.

A complete calculation pipeline requires an `EffectPlanner`, `EffectApplier`, `EffectAccumulatorFactory`, and `EffectAccumulatorFinalizer`. A project using the generic planning algorithm implements `EffectPlanningPolicy` and constructs `EffectExecutionPlanner`.

Build-level calculation requires `BuildEffectCollector`. Stateful current-build caching requires `BuildCalculationCore`. Final output comparison requires `CalculationOutputComparator`. Candidate descriptions require `BuildCandidateFactory`.

`PassiveNodeProvider` is optional and is only needed when the concrete project has a passive-node lookup layer. Text parsers are optional when item data is constructed or deserialized through another interface. The stateful core is optional when the application only needs one-shot calculations.

---

<a id="section-17"></a>

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
        EffectCalculator,
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

`MyDefinitionProvider` owns the movement-speed definition. The value that later appears as `movement_speed_definition` will be borrowed from this provider rather than appearing as an unexplained variable.

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

The factory creates `MyAccumulator`, `MyEffectApplier` mutates it, and `MyFinalizer` produces `MyFinalStats`.

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
        EffectCalculator::new(
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

<a id="section-18"></a>

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

Public result aliases used by calculator, collector, build, and comparison methods are re-exported through the same facade. External code should not need paths such as `exile_core::effect::calculation::...` or `exile_core::item::model::...`.

Some constructors are intentionally crate-private because they certify a relationship between values. `EffectExecutionPlan` must come from a planner. `EffectSelectionRejection` must be created while planner winners are known. `EffectCalculationOutput` must pair an output with the plan that produced it. `CalculationComparison` must pair two outputs with a difference produced by the comparator.

External code uses the semantic public paths instead. It obtains execution plans from `EffectPlanner::plan`, detailed outputs from `EffectCalculator`, and comparisons from `CalculationComparison::between` or `CalculationComparisonRunner`.

---

<a id="section-19"></a>

## 19. Known Design Questions and Open Issues

### Recoverable calculation failure

The current accumulator ownership contract is defined in [Section 9](#section-9), and the lower-level distinction between `apply_all` and `apply_all_owned` is defined in [Section 8](#section-8).

An open issue should decide whether the consumption-only calculator API is sufficient or whether a separate recoverable path is required. Recovery may be useful when accumulator construction is expensive, when diagnostics need intermediate state, or when callers need to reset and reuse calculation resources.

Possible alternatives include returning the accumulator inside an error, exposing a separate recoverable calculation method, or introducing snapshot-based rollback. Returning the value after an application failure would expose its partially mutated state unless an additional rollback mechanism were implemented. Requiring `Accumulator: Clone` would simplify snapshots but would impose a potentially expensive and unnecessary bound on every integration.

Until that design is resolved, callers that require retries should retain the source input needed to construct a fresh accumulator.

### Transactional effect application

The current mutation model is intentionally non-transactional, as described in [Section 8](#section-8). A separate issue may evaluate whether transactional application belongs in `exile-core`, should be provided by game-specific accumulators, or should remain outside the library.

### Floating-point comparison policy

The current literal floating-point behavior is documented in [Section 13](#section-13). A future issue may consider whether the library should provide an optional approximate-comparison helper, but game-specific tolerance rules should not silently replace the existing exact semantics.

---
