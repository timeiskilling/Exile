pub trait Game {
    type ItemBase;
    type ItemState;

    type ModifierDefinition;
    type ModifierInstance;
}
