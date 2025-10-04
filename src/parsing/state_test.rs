use super::{ARENA_REUSE_UPPER_BOUND, ArenaLease};
use assert_matches::assert_matches;

mod arena_lease_acquire {
    use super::*;

    #[test]
    fn should_return_guard_from_pool_when_zero_capacity_requested_then_allow_string_allocation() {
        let min_capacity = 0;

        let lease = ArenaLease::acquire(min_capacity);

        assert_matches!(lease, ArenaLease::Guard(guard) => {
            let stored = guard.alloc_str("alpha");
            assert_eq!(stored, "alpha");
        });
    }

    #[test]
    fn should_return_pooled_guard_when_within_reuse_upper_bound_then_allocate_strings() {
        let min_capacity = ARENA_REUSE_UPPER_BOUND;

        let lease = ArenaLease::acquire(min_capacity);

        assert_matches!(lease, ArenaLease::Guard(guard) => {
            let stored = guard.alloc_str("beta");
            assert_eq!(stored, "beta");
        });
    }

    #[test]
    fn should_allocate_owned_arena_when_capacity_exceeds_upper_bound_then_allocate_strings() {
        let min_capacity = ARENA_REUSE_UPPER_BOUND + 1;

        let lease = ArenaLease::acquire(min_capacity);

        assert_matches!(lease, ArenaLease::Owned(arena) => {
            let stored = arena.alloc_str("gamma");
            assert_eq!(stored, "gamma");
        });
    }
}

mod arena_lease_deref {
    use super::*;

    #[test]
    fn should_allow_shared_access_when_guard_is_dereferenced_then_allocate_string() {
        let lease = ArenaLease::acquire(1);

        let stored = lease.alloc_str("shared");

        assert_eq!(stored, "shared");
    }

    #[test]
    fn should_allow_mutations_when_guard_is_mutably_dereferenced_then_prepare_and_allocate() {
        let mut lease = ArenaLease::acquire(8);

        lease.prepare(0);
        let stored = lease.alloc_str("mutated");

        assert_eq!(stored, "mutated");
    }

    #[test]
    fn should_allow_shared_access_when_arena_is_owned_then_allocate_string() {
        let lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        let stored = lease.alloc_str("owned");

        assert_eq!(stored, "owned");
    }

    #[test]
    fn should_allow_mutations_when_arena_is_owned_then_resize_and_allocate() {
        let mut lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        lease.prepare(ARENA_REUSE_UPPER_BOUND + 2);
        let stored = lease.alloc_str("resized");

        assert_eq!(stored, "resized");
    }
}

mod arena_lease_debug {
    use super::*;

    #[test]
    fn should_format_guard_variant_then_emit_debug_identifier() {
        let lease = ArenaLease::acquire(0);

        assert_eq!(format!("{:?}", lease), "ArenaLease::Guard");
    }

    #[test]
    fn should_format_owned_variant_then_emit_debug_identifier() {
        let lease = ArenaLease::acquire(ARENA_REUSE_UPPER_BOUND + 1);

        assert_eq!(format!("{:?}", lease), "ArenaLease::Owned");
    }
}
