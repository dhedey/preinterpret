use preinterpret::*;

fn main() {
    run!(
        let obj = %{};
        // This *should* work, in the sense that replace takes an assignee...
        // And obj.unused_key as an assignee should auto-create an entry.
        // 
        // But it's a limitation of the current implementation: we don't know
        // we're mapping an assignee because we can't resolve the replace
        // interface yet without knowing the type of the value. So we map
        // a late-bound mutable value.
        //
        // Or in more detail:
        // - To resolve the receiver type, we request a late-bound mutable value.
        // - This starts by attempting to map a mutable access.
        // - But since the key doesn't exist, the mutable mapping fails.
        // - We then fall back to shared access
        // - Then when we try to map to assignee, we push the reason_not_mutable
        //
        // All that said, it would be kinda weird if this *did* work, so it
        // doesn't need fixing.
        obj.unused_key.replace(123);
        %[].assert_eq(obj.unused_key, 123);
        // Ensure the line above isn't a last-use owned, so resolves a ref
        let _ = obj;
    );
}
