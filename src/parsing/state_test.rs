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

mod arena_lease_deref_tests {
    use super::*;

    #[test]
    fn when_guard_is_dereferenced_it_should_allow_shared_access() {
        // Arrange
        let lease = ArenaLease::acquire(1);

        // Act
        let stored = lease.alloc_str("shared");

        // Assert
        assert_eq!(stored, "shared");
    }

    #[test]
    fn when_guard_is_mutably_dereferenced_it_should_allow_mutations() {
        // Arrange
        let mut lease = ArenaLease::acquire(8);

        // Act
        lease.prepare(0);
        let stored = lease.alloc_str("mutated");

        // Assert
        assert_eq!(stored, "mutated");
    }

    #[test]
    fn when_owned_is_dereferenced_it_should_allow_shared_access() {
        // Arrange
        let lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        let stored = lease.alloc_str("owned");

        // Assert
        assert_eq!(stored, "owned");
    }

    #[test]
    fn when_owned_is_mutably_dereferenced_it_should_allow_mutations() {
        // Arrange
        let mut lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        lease.prepare(ARENA_REUSE_UPPER_BOUND + 2);
        let stored = lease.alloc_str("resized");

        // Assert
        assert_eq!(stored, "resized");
    }
}
