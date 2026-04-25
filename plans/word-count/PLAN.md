# Word Count Stats Plan

**Author:** pi agent
**Date:** 2026-04-25
**Status:** Draft
**Version:** 1.0

---

## Purpose

Track and display the total number of words transcribed across all sessions. Provides users with a simple metric showing how much typing they've saved by using Clanker Yap. Non-intrusive, always visible, persists across sessions.

---

## Scope

### In Scope

| Item | Priority | Notes |
|------|----------|-------|
| Add `total_words` field to Settings | P0 | Persisted, starts at 0 |
| Increment word count on each transcription save | P0 | Words in new transcription added to total |
| Expose word count via `get_settings` | P0 | No new commands needed |
| Display word count in frontend | P0 | Minimal, fits theme, no UI bloat |

### Out of Scope

- Words per individual transcription entry (shown in history)
- Session vs all-time breakdown
- Reset capability
- Charts, graphs, or detailed analytics
- Any change to app behavior or recording flow

---

## What Already Exists

| Component | Location | Status |
|-----------|----------|--------|
| Settings struct | `domain/settings.rs` | ✅ Exists |
| Settings persistence | `infrastructure/persistence/settings_repo.rs` | ✅ Exists |
| Settings DTO | `presentation/dto.rs` | ✅ Exists |
| Transcription entity | `domain/transcription.rs` | ✅ Exists |
| Save transcription use case | `application/use_cases/transcription.rs` | ✅ Exists |
| Frontend history display | `src/main.js` | ✅ Exists |

### Existing Patterns to Follow

Settings persistence pattern from `domain/settings.rs`:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // existing fields...
    // new: total_words: u64,
}
```

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| None | N/A | Standalone feature |

---

## Implementation Plan

### Phase Order

| Phase | Description | Depends On |
|-------|-------------|-------------|
| 0 | Add `total_words` to Settings domain + persistence | — |
| 1 | Update word count in transcription save use case | Phase 0 |
| 2 | Expose word count in frontend | Phase 1 |

### Phase Details

#### Phase 0 — Add `total_words` to Settings

**Purpose:** Persist total word count across sessions.

**Scope:**
- [ ] Add `total_words: u64` field to `Settings` struct in `domain/settings.rs`
- [ ] Add `total_words` to `SettingsResponse` DTO in `presentation/dto.rs`
- [ ] Verify settings load/save works with new field (backwards compatible)

**Out of Scope:**
- Word counting logic (Phase 1)
- Frontend display (Phase 2)

**Files to Modify:**
- `src-tauri/src/domain/settings.rs` — Add `total_words` field with default `0`
- `src-tauri/src/presentation/dto.rs` — Add `total_words` to `SettingsResponse`

**Context Files to Read:**
- `src-tauri/src/domain/settings.rs` — Existing Settings structure

---

#### Phase 1 — Update word count on save

**Purpose:** Increment total words each time a transcription is saved.

**Scope:**
- [ ] In `application/use_cases/transcription.rs` `save_transcription()`, count words in transcription text
- [ ] Load settings, increment `total_words`, save settings
- [ ] Create helper function `count_words(text: &str) -> usize` in domain

**Out of Scope:**
- Frontend display
- Any other stats

**Files to Modify:**
- `src-tauri/src/application/use_cases/transcription.rs` — Update save logic to increment word count
- (Possibly new) `src-tauri/src/domain/transcription.rs` — Add `count_words()` helper if needed

**Context Files to Read:**
- `src-tauri/src/application/use_cases/transcription.rs` — Current save logic

---

#### Phase 2 — Display word count in frontend

**Purpose:** Show total words transcribed in the UI.

**Scope:**
- [ ] Add word count display to `src/main.js` 
- [ ] Styling to match existing dark theme, minimal footprint
- [ ] Update on each transcription and on settings load

**Out of Scope:**
- Any layout changes beyond adding the display
- Interactive features (reset, etc.)

**Files to Modify:**
- `src/main.js` — Add word count display element and update logic
- `src/style.css` — Minimal styling if needed

**Context Files to Read:**
- `src/main.js` — Current UI structure and settings handling
- `src/style.css` — Current styling

---

## API Endpoints / Commands

No new commands needed. Word count is returned via existing `get_settings` command.

### `get_settings` (existing, extended)

**Response:**
```rust
struct SettingsResponse {
    // existing fields...
    pub total_words: u64,  // NEW: total words transcribed
}
```

---

## File Structure

Legend: ✅ exists, 🔧 modify, 🆕 new

```
src-tauri/
├── src/
│   ├── domain/
│   │   └── settings.rs         🔧 Add total_words field
│   ├── application/
│   │   └── use_cases/
│   │       └── transcription.rs  🔧 Increment word count on save
│   └── presentation/
│       └── dto.rs               🔧 Add total_words to response
src/
├── main.js                     🔧 Add word count display
└── style.css                   🔧 Minimal styling if needed
```

---

## New Dependencies

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| None | — | — | Not needed |

---

## Testing Strategy

### Unit Tests
- [ ] `count_words()` helper: empty string, single word, multiple words, whitespace handling
- [ ] `total_words` default value is 0
- [ ] Settings serialization/deserialization with `total_words`

### Integration Tests
- [ ] Full flow: save transcription, verify settings `total_words` updated

### Smoke Tests
- [ ] App launches without panic
- [ ] Settings load/save works with new field
- [ ] Word count displays in UI

---

## Rollout Plan

| Phase | Scope | Verification |
|-------|-------|--------------|
| 0 | Settings domain + DTO | `cargo test`, `cargo clippy`, `cargo fmt` |
| 1 | Word count increment | `cargo test`, end-to-end save/load |
| 2 | Frontend display | Visual check, hot reload dev |

---

## Related Documents

- `AGENTS.md` — Architecture principles (slim frontend, heavy backend)
- `src-tauri/src/domain/settings.rs` — Settings structure reference
- `src-tauri/src/application/use_cases/transcription.rs` — Save logic reference
- `src/main.js` — Frontend reference

---

## Checklist

### Phase 0
- [ ] Add `total_words: u64` to `Settings` struct with default `0`
- [ ] Add `total_words: u64` to `SettingsResponse` DTO
- [ ] Tests pass

### Phase 1
- [ ] Implement `count_words()` helper
- [ ] Update `save_transcription()` to increment word count
- [ ] Tests pass

### Phase 2
- [ ] Add word count display to frontend
- [ ] Style matches existing theme

### Verification
- [ ] App launches without panic
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` passes
- [ ] Word count persists across app restarts

---

## Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-25 | pi agent | Initial draft |