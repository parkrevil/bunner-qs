use super::{ARENA_REUSE_UPPER_BOUND, ArenaLease};

mod arena_lease_acquire {
    use super::*;

    #[test]
    fn should_return_guard_from_pool_when_zero_capacity_requested_then_allow_string_allocation() {
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
    fn should_return_pooled_guard_when_within_reuse_upper_bound_then_allocate_strings() {
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
    fn should_allocate_owned_arena_when_capacity_exceeds_upper_bound_then_allocate_strings() {
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
    fn should_allow_shared_access_when_guard_is_dereferenced_then_allocate_string() {
        // Arrange
        let lease = ArenaLease::acquire(1);

        // Act
        let stored = lease.alloc_str("shared");

        // Assert
        assert_eq!(stored, "shared");
    }

    #[test]
    fn should_allow_mutations_when_guard_is_mutably_dereferenced_then_prepare_and_allocate() {
        // Arrange
        let mut lease = ArenaLease::acquire(8);

        // Act
        lease.prepare(0);
        let stored = lease.alloc_str("mutated");

        // Assert
        assert_eq!(stored, "mutated");
    }

    #[test]
    fn should_allow_shared_access_when_arena_is_owned_then_allocate_string() {
        // Arrange
        let lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        let stored = lease.alloc_str("owned");

        // Assert
        assert_eq!(stored, "owned");
    }

    #[test]
    fn should_allow_mutations_when_arena_is_owned_then_resize_and_allocate() {
        // Arrange
        let mut lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        // Act
        lease.prepare(ARENA_REUSE_UPPER_BOUND + 2);
        let stored = lease.alloc_str("resized");

        // Assert
        assert_eq!(stored, "resized");
    }
}
