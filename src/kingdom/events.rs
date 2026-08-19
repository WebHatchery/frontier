//! Deterministic kingdom-event consequences used after an expedition.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KingdomEventKind {
    Plague,
    Thieves,
    Traders,
    Festival,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KingdomEventOutcome {
    pub message: String,
    pub gold_delta: i32,
    pub supplies_delta: i32,
    pub morale_delta: i32,
    pub stress_delta: i32,
}

pub fn evaluate_kingdom_event(kind: KingdomEventKind, gold: i32) -> KingdomEventOutcome {
    match kind {
        KingdomEventKind::Plague => KingdomEventOutcome {
            message: "Plague: morale fell and the roster gained stress.".to_string(),
            gold_delta: 0,
            supplies_delta: 0,
            morale_delta: -6,
            stress_delta: 6,
        },
        KingdomEventKind::Thieves => {
            let stolen = gold.min(25);
            KingdomEventOutcome {
                message: format!("Thieves: {} gold was stolen from the stores.", stolen),
                gold_delta: -stolen,
                supplies_delta: 0,
                morale_delta: 0,
                stress_delta: 0,
            }
        }
        KingdomEventKind::Traders if gold >= 15 => KingdomEventOutcome {
            message: "Traders: paid 15 gold for 25 supplies.".to_string(),
            gold_delta: -15,
            supplies_delta: 25,
            morale_delta: 0,
            stress_delta: 0,
        },
        KingdomEventKind::Traders => KingdomEventOutcome {
            message: "Traders: a small debt was forgiven for 10 gold.".to_string(),
            gold_delta: 10,
            supplies_delta: 0,
            morale_delta: 0,
            stress_delta: 0,
        },
        KingdomEventKind::Festival => KingdomEventOutcome {
            message: "Festival: morale rose and adventurers shed a little stress.".to_string(),
            gold_delta: 0,
            supplies_delta: 0,
            morale_delta: 8,
            stress_delta: -3,
        },
    }
}

#[cfg(test)]
mod tests;
