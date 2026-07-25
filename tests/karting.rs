#![allow(unused)]

use enum_info::enum_info;

#[test]
fn test_racers() {
	// Should have 8 variants.
	#[enum_info]
	enum KartRacers {
		Katty,     // From Mario.
		Jax,       // From Luigi.
		Jo,        // From Toad.
		Mayo,      // From Peach.
		Guest1,    // -> ???
		DK,        // -> ???
		Bowser,    // -> ???
		Wario      // -> ???
	}
	
	assert_eq!(KartRacers::variant_count(), 8);
}