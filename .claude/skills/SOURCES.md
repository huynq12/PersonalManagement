# Vendored skills

Copied unmodified from the upstream repositories below. To update a skill, re-copy it from a newer
commit and update this table.

| Skill directory | Upstream | Upstream path | Commit | License |
|---|---|---|---|---|
| `test-driven-development` | https://github.com/obra/superpowers | `skills/test-driven-development` | `22f0b463e7a5e579904dd6211a520321035526f7` | MIT |
| `systematic-debugging` | https://github.com/obra/superpowers | `skills/systematic-debugging` | `22f0b463e7a5e579904dd6211a520321035526f7` | MIT |
| `verification-before-completion` | https://github.com/obra/superpowers | `skills/verification-before-completion` | `22f0b463e7a5e579904dd6211a520321035526f7` | MIT |
| `rust-skills` | https://github.com/leonardomso/rust-skills | `SKILL.md`, `rules/` | `fd2a861ab0406a4ac536a55274d14ea6fd1ca9c9` | MIT |
| `vercel-react-best-practices` | https://github.com/vercel-labs/agent-skills | `skills/react-best-practices` | `1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62` | MIT (declared in `SKILL.md`; the repository has no LICENSE file) |
| `vercel-composition-patterns` | https://github.com/vercel-labs/agent-skills | `skills/composition-patterns` | `1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62` | MIT (declared in `SKILL.md`; the repository has no LICENSE file) |

Notes:

- The superpowers skills refer to each other as `superpowers:<name>`; here they are available by their bare names.
- `rust-skills` targets Rust 1.96; check a rule against the installed toolchain before applying newer language features.
- Many `vercel-react-best-practices` rules target Next.js and do not apply to this Vite SPA.
