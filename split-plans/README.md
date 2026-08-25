# split-plans/

One markdown file per source file that still exceeds the 130-line budget.
Each plan is written by an analysis pass BEFORE any code moves, so the
split is reviewed as a decision rather than discovered as a diff.

A plan answers four questions:

1. **Why is this file long?** Genuine multi-responsibility, or one
   irreducible thing (a big enum, a required declaration list)? If it is
   irreducible, the plan says so and the file keeps a documented exception
   instead of being mangled.
2. **What are the seams?** The cohesive groups inside the file, with the
   exact line ranges that move.
3. **What does the public surface become?** Slint re-exports from the
   original path, Rust `mod` + `pub use` — importers must not change.
4. **What can break?** Element-id scoping in Slint, two-way bindings,
   private-property access, `super::` paths in Rust.

These are working documents. Once a plan is executed and the file is under
budget, the plan stays as the record of why the split is shaped that way.
