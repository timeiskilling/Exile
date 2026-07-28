mod effect_calculation_output;
mod effect_calculator;

pub use effect_calculation_output::EffectCalculationOutput;

pub use effect_calculator::{
    EffectCalculationDetailedResult, EffectCalculationError,
    EffectCalculationFromInputDetailedResult, EffectCalculationFromInputError,
    EffectCalculationFromInputResult, EffectCalculationResult, EffectCalculator,
};
