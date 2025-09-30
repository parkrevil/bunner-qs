use super::{ARENA_REUSE_UPPER_BOUND, ArenaLease};

mod arena_lease_acquire {
    use super::*;

    #[test]
    fn returns_guard_from_pool_for_zero_capacity_request() {
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
    fn returns_pooled_guard_within_reuse_upper_bound() {
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
    fn allocates_owned_arena_when_capacity_exceeds_upper_bound() {
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

mod arena_lease_deref {
    use super::*;

    #[test]
    fn allows_shared_access_when_guard_is_dereferenced() {
        // Arrange
        let lease = ArenaLease::acquire(1);

        // Act
        let stored = lease.alloc_str("shared");

        // Assert
        assert_eq!(stored, "shared");
    }

    #[test]
    fn allows_mutations_when_guard_is_mutably_dereferenced() {
        // Arrange
        let mut lease = ArenaLease::acquire(8);

        // Act
        lease.prepare(0);
        let stored = lease.alloc_str("mutated");

        // Assert
        assert_eq!(stored, "mutated");
    }

    #[test]
    fn allows_shared_access_for_owned_arena() {
        // Arrange
        let lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        let stored = lease.alloc_str("owned");

        // Assert
        assert_eq!(stored, "owned");
    }

    #[test]
    fn allows_mutations_for_owned_arena() {
        // Arrange
        let mut lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        lease.prepare(ARENA_REUSE_UPPER_BOUND + 2);
        let stored = lease.alloc_str("resized");

        // Assert
        assert_eq!(stored, "resized");
    }
}
