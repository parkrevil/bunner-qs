use super::{ARENA_REUSE_UPPER_BOUND, ArenaLease};

mod arena_lease_acquire_tests {
    use super::*;

    #[test]
    fn when_min_capacity_is_zero_it_returns_guard_from_pool() {
        // Arrange
        let min_capacity = 0;

        // Act
        let lease = ArenaLease::acquire(min_capacity);

        // Assert
        match lease {
            ArenaLease::Guard(guard) => {
                let stored = guard.alloc_str("alpha");
                assert_eq!(stored, "alpha");
            }
            ArenaLease::Owned(_) => panic!("expected pooled guard"),
        }
    }

    #[test]
    fn when_min_capacity_is_within_reuse_upper_bound_it_uses_guard() {
        // Arrange
        let min_capacity = ARENA_REUSE_UPPER_BOUND;

        // Act
        let lease = ArenaLease::acquire(min_capacity);

        // Assert
        match lease {
            ArenaLease::Guard(guard) => {
                let stored = guard.alloc_str("beta");
                assert_eq!(stored, "beta");
            }
            ArenaLease::Owned(_) => panic!("expected pooled guard"),
        }
    }

    #[test]
    fn when_min_capacity_exceeds_upper_bound_it_allocates_owned_arena() {
        // Arrange
        let min_capacity = ARENA_REUSE_UPPER_BOUND + 1;

        // Act
        let lease = ArenaLease::acquire(min_capacity);

        // Assert
        match lease {
            ArenaLease::Owned(arena) => {
                let stored = arena.alloc_str("gamma");
                assert_eq!(stored, "gamma");
            }
            ArenaLease::Guard(_) => panic!("expected owned arena"),
        }
    }
}
