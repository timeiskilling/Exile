mod calculation_baseline;
mod calculation_comparison;
mod calculation_comparison_runner;
mod calculation_output_comparator;
mod numeric_stat_difference;
mod stat_value_difference;

pub use calculation_baseline::CalculationBaseline;
pub use calculation_comparison::CalculationComparison;

pub use calculation_comparison_runner::{
    CalculationBaselineFromInputResult, CalculationBaselineOutput, CalculationComparisonError,
    CalculationComparisonFromInputError, CalculationComparisonFromInputResult,
    CalculationComparisonOutput, CalculationComparisonRunner, CalculationFromInputErrorFor,
    CalculationOutputDifference, CandidateComparisonError, CandidateComparisonFromInputError,
    CandidateComparisonFromInputResult, FinalizedCalculationOutput,
};

pub use calculation_output_comparator::CalculationOutputComparator;
pub use numeric_stat_difference::NumericStatDifference;
pub use stat_value_difference::StatValueDifference;
