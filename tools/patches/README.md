# Makepad compatibility patch

Rinx's pinned Makepad compatibility revision already contains this fix, alongside
the shared Markdown crates. Normal builds do not need to modify Cargo's checkout.
To verify the selected dependency:

```sh
python3 tools/apply-makepad-patches.py --check
cargo build --release --locked --bin rinx
```

The tool finds the single Makepad checkout selected by Cargo.lock. Without
`--check`, it can still apply the patch to the former SDK revision and invalidate
its compiled dependency. It leaves an already patched checkout alone.

`makepad-metal-dropped-draw-lists.patch` fixes a Metal lifetime bug in Makepad
47837267: a cached parent draw list may still reference a child belonging to a
closed widget. Both the texture-upload traversal and render traversal must skip
freed IDs, including pool slots reused for another widget. Otherwise they visit
the replacement widget through an old generation and log errors or render the
wrong content. This uses Makepad's existing `is_id_freed` check.

The native desktop navigation regression in
`tools/wechat-ux/live/native_article_markdown.py --desktop-entry` repeatedly
switches between cached direct and group chats, then saves and restores an
article draft. It rejects draw-list errors in every process log. The patch is
retained here to verify the compatibility revision and support older checkouts.
Current Makepad `dev` already includes guards for the reproduced stale-child
failure; the Markdown upstream PR does not resubmit that framework fix.
