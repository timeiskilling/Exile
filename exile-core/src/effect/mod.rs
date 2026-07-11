#[derive(Debug, Clone)]
pub enum Effect {
    Add {
        stat: StatId,
        value: f64,
    },

    Multiply {
        stat: StatId,
        factor: f64,
    },

    Set {
        stat: StatId,
        value: f64,
    },

    Clamp {
        stat: StatId,
        min: Option<f64>,
        max: Option<f64>,
    },

    Custom {
        mechanic: MechanicId,
        payload: DataValue,
    },
}