use std::cmp::Ordering;

use super::*;

/// Chopping wood adds 1 wood.
#[test]
fn test_chop() {
    let expected_state = GameState {
        wood: Wood::new(1),
        ..Default::default()
    };

    let mut state = GameState::default();
    state.chop();

    assert_eq!(state, expected_state);
}

// Can sell wood for gold.
#[test]
fn test_sell_wood() {
    let mut state = GameState::default();

    // Selling wood when you don't have enough is a noop.
    state
        .wood
        .update(ResourceDiff::new(((WOOD_PER_GOLD - 1) as i128) * 1000));
    let expected_state = state.clone();
    state.sell_wood();
    assert_eq!(state, expected_state);

    // If there is enough then selling turns wood to gold
    state.wood.increment();
    let expected_state = GameState {
        gold: Gold::new(1),
        ..Default::default()
    };
    state.sell_wood();
    assert_eq!(state, expected_state);
}

// Can spend gold to hire a miner.
#[test]
fn test_hire_miner() {
    let mut state = GameState::default();

    // Hiring a miner when you don't have enough gold is a noop.
    state
        .gold
        .update(ResourceDiff::new(((MINER_GOLD_COST - 1) as i128) * 1000));
    let expected_state = state.clone();
    state.hire_miner();
    assert_eq!(state, expected_state);

    // If there is enough then hiring turns gold to a miner
    state.gold.increment();
    let expected_state = GameState {
        miners: Miner::new(1),
        ..Default::default()
    };
    state.hire_miner();
    assert_eq!(state, expected_state);
}

// Can freely hire a lumberjack
#[test]
fn test_hire_lumberjack() {
    let expected_state = GameState {
        lumberjacks: Lumberjack::new(1),
        ..Default::default()
    };
    let mut state = GameState::default();
    state.hire_lumberjack();
    assert_eq!(state, expected_state);
}

/// Furnaces convert 1 wood into 1 energy per second.
#[test]
fn test_update_furnace() {
    let mut state = GameState {
        furnaces: Furnace::new(1),
        ..Default::default()
    };

    // Without wood furnaces cannot operate.
    for _ in 0..100 {
        state.update();
        assert_eq!(state.energy.raw_amount(), 0);
    }

    // With wood, converts 1 per second
    state.wood.increment();
    for _ in 0..99 {
        state.update();
        assert_eq!(state.energy.raw_amount(), 0);
        // There was only 1 wood to start with so after
        // subtraction there is milli wood left but no whole wood.
        assert_eq!(state.wood.raw_amount(), 0);
    }
    state.update();
    let expected_state = GameState {
        furnaces: Furnace::new(1),
        energy: Energy::new(1),
        ..Default::default()
    };
    assert_eq!(state, expected_state);

    // 3 furnaces can convert 1 wood in 34 updates
    state = GameState {
        furnaces: Furnace::new(3),
        wood: Wood::new(1),
        ..Default::default()
    };
    for _ in 0..33 {
        state.update();
        assert_eq!(state.energy.raw_amount(), 0);
        assert_eq!(state.wood.raw_amount(), 0);
    }
    state.update();
    let expected_state = GameState {
        furnaces: Furnace::new(3),
        energy: Energy::new(1),
        ..Default::default()
    };
    assert_eq!(state, expected_state);

    // 100 furnaces can convert 1 wood in a single update
    state = GameState {
        furnaces: Furnace::new(100),
        wood: Wood::new(2),
        ..Default::default()
    };
    state.update();
    let expected_state = GameState {
        furnaces: Furnace::new(100),
        energy: Energy::new(1),
        wood: Wood::new(1),
        ..Default::default()
    };
    assert_eq!(state, expected_state);

    state.update();
    let expected_state = GameState {
        furnaces: Furnace::new(100),
        energy: Energy::new(2),
        ..Default::default()
    };
    assert_eq!(state, expected_state);

    // test with a fractional amount of wood
    state = GameState {
        furnaces: Furnace::new(70),
        wood: Wood::new_milli(600),
        ..Default::default()
    };
    state.update();
    let expected_state = GameState {
        furnaces: Furnace::new(70),
        energy: Energy::new_milli(600),
        ..Default::default()
    };
    assert_eq!(state, expected_state);
}

/// Lumber jacks convert 1 gold into 1 wood per second.
#[test]
fn test_update_lumberjack() {
    let mut state = GameState {
        lumberjacks: Lumberjack::new(1),
        ..Default::default()
    };

    // Without gold lumberjacks cannot operate.
    for _ in 0..100 {
        state.update();
        assert_eq!(state.wood.raw_amount(), 0);
    }

    // With gold, converts 1 per second
    state.gold.increment();
    for _ in 0..99 {
        state.update();
        assert_eq!(state.wood.raw_amount(), 0);
        assert_eq!(state.gold.raw_amount(), 0);
    }
    state.update();
    let expected_state = GameState {
        lumberjacks: Lumberjack::new(1),
        wood: Wood::new(1),
        ..Default::default()
    };
    assert_eq!(state, expected_state);
}

// Factories produce one miner per second using energy
#[test]
fn test_update_factory() {
    let mut state = GameState {
        factories: Factory::new(1),
        ..Default::default()
    };

    // Without energy factories cannot operate.
    for _ in 0..100 {
        state.update();
        assert_eq!(state.miners.raw_amount(), 0);
    }

    // With energy, converts 1 per second
    state.energy.increment();
    for _ in 0..99 {
        state.update();
        assert_eq!(state.miners.raw_amount(), 0);
        assert_eq!(state.energy.raw_amount(), 0);
    }
    state.update();
    let expected_state = GameState {
        factories: Factory::new(1),
        miners: Miner::new(1),
        ..Default::default()
    };
    assert_eq!(state, expected_state);
}

// Gold grows linearly with one miner
#[test]
fn test_linear_growth() {
    let mut state = GameState {
        miners: Miner::new(1),
        ..Default::default()
    };

    let mut gold_amount = Vec::with_capacity(100);
    for _ in 0..100 {
        gold_amount.push(state.gold.raw_amount());
        for _ in 0..100 {
            state.update();
        }
    }

    for (x, y) in gold_amount.into_iter().enumerate() {
        let expected = x as u128;
        assert_eq!(y, expected);
    }
}

// The state can grow gold quadratically by having
// lumberjacks make wood which feeds the furnace that makes energy
// to operate the factory which produces miner that make gold.
#[test]
fn test_quadratic_growth() {
    let mut state = GameState {
        miners: Miner::new(1),
        lumberjacks: Lumberjack::new(1),
        furnaces: Furnace::new(1),
        factories: Factory::new(1),
        ..Default::default()
    };

    let mut gold_amount = Vec::with_capacity(100);
    for _ in 0..100 {
        gold_amount.push(state.gold.raw_amount());
        for _ in 0..100 {
            state.update();
        }
    }

    for (x, y) in gold_amount.into_iter().enumerate().skip(1) {
        let expected = (x as u128) * (x as u128 - 1) / 2;
        assert_eq!(y, expected);
    }
}

// Banks can grow gold exponentially by itself
#[test]
fn test_exponential_growth() {
    let mut state = GameState {
        gold: Gold::new(1),
        banks: Bank::new(1),
        ..Default::default()
    };

    let mut gold_amount = Vec::with_capacity(100);
    for _ in 0..100 {
        gold_amount.push(state.gold.raw_milli_amount());
        state.update();
    }

    for (&y2, &y1) in gold_amount.iter().skip(1).zip(gold_amount.iter()) {
        // Approximately equal to a geometric series with common ratio 1.01.
        // For ease of computing with integers an rewrite as
        // 1000 * (y2 - y1) / y1 = 9 or 10
        let r = 1000 * (y2 - y1) / y1;
        assert!(r == 9 || r == 10);
    }
}

// The circle of life:
// Miners mine the gold we use to pay the lumberjacks;
// Advertisers attract more lumberjacks with the size of our coffers;
// Monsters eat the lumberjacks;
#[test]
fn test_circular_motion() {
    let mut state = GameState {
        miners: Miner::new(1000),
        monsters: Monster::new(700),
        recruiters: Recruiter::new(1),
        lumberjacks: Lumberjack::new(1000),
        gold: Gold::new(1100),
        ..Default::default()
    };

    let mut gold_amount = Vec::with_capacity(100);
    let mut lumberjack_amount = Vec::with_capacity(100);
    for _ in 0..625 {
        gold_amount.push(state.gold.raw_amount());
        lumberjack_amount.push(state.lumberjacks.raw_amount());
        state.update();
    }

    for (x, y) in gold_amount.into_iter().zip(lumberjack_amount) {
        // The radius of the circle is approximately 400
        let r = isqrt((x as i128 - 700).pow(2) + (y as i128 - 1000).pow(2));
        assert!((399..=412).contains(&r));
    }
}

fn isqrt(x: i128) -> i128 {
    if x == 1 {
        return 1;
    }
    if x < 1 {
        return 0;
    }
    binary_search(1, x, x)
}

fn binary_search(mut a: i128, mut b: i128, x: i128) -> i128 {
    while a != b - 1 {
        let c = (a + b) / 2;
        let c_sq = c * c;
        match c_sq.cmp(&x) {
            Ordering::Equal => return c,
            Ordering::Less => a = c,
            Ordering::Greater => b = c,
        }
    }
    a
}
