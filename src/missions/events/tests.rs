use super::*;

#[test]
fn twisted_path_choices_apply_their_complete_deltas() {
    let event = Event::twisted_path();

    assert_eq!(
        evaluate_choice(&event.choices[0]),
        EventOutcomeDelta {
            stress: 5,
            skip_node: true,
            ..Default::default()
        }
    );
    assert_eq!(
        evaluate_choice(&event.choices[1]).combat_enemy,
        Some("shadow_wolf".to_string())
    );
    assert_eq!(
        evaluate_choice(&event.choices[2]),
        EventOutcomeDelta {
            stress: 10,
            knowledge: 5,
            ..Default::default()
        }
    );
}

#[test]
fn every_event_choice_is_deterministic_and_preserves_healing_sign() {
    let events = [
        Event::ancient_marker(),
        Event::forest_shrine(),
        Event::twisted_path(),
    ];

    for event in events {
        for choice in event.choices {
            let first = evaluate_choice(&choice);
            assert_eq!(first, evaluate_choice(&choice));
        }
    }
    assert_eq!(
        evaluate_choice(&Event::forest_shrine().choices[0]).stress,
        -10
    );
}
