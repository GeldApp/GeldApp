# Weekly release review — 2026-11-15

Independent Terra reviewed `2c8dc2139b1a045ed073af69215f98f98bd41145` and approved it for a draft PR and live CI. The earlier tag-target race is addressed by atomic ref creation at the checked SHA, immutable version-tag protection, exact ref readback and only then publication. Existing/conflicting tags fail closed.

The workflow uses a frozen, validated main snapshot; separate head checks cannot atomically guarantee that main will remain unchanged until the publish call. The docs state that limit and describe manual recovery if tag creation succeeds but publication fails. Root applied tag ruleset `27325536` and verified active update/deletion restrictions with no bypass actors. The public workflow readback cannot observe hidden bypass actors; trusted repository administrators must preserve that policy.

Worker validation includes actionlint, ruleset fixtures/live readback, version helper annotated/lightweight tag fixtures and conflict fixtures. These are not a live scheduled publication test. Main pushes remain draft-only; automatic publication is scheduled for Monday 08:00 UTC and still requires all checks and security gates.
