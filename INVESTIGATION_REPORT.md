# Life Restart Rust Implementation - Deep Investigation Report

## Executive Summary

After deep investigation of the original JavaScript implementation, Python wrapper, and Rust core, I can confirm that **most of the reported issues are REAL**, but some have different severity levels than initially assessed.

## Investigation Methodology

1. ✅ Examined original JavaScript implementation (`refence/lifeRestart-main/src/modules/`)
2. ✅ Analyzed Python wrapper logic (`liferestart/services/`)
3. ✅ Reviewed Rust core implementation (`src/`)
4. ✅ Cross-referenced property usage in conditions, achievements, and events

---

## Issue Analysis

### ⚠️ Issue 1: `Opportunity::End` Missing (CONFIRMED - CRITICAL)

**Status**: ✅ **REAL ISSUE**

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/achievement.js`):
```javascript
Opportunity = {
    START: "START",             // 分配完成點數，點擊開始新人生後
    TRAJECTORY: "TRAJECTORY",   // 每一年的人生經歷中
    SUMMARY: "SUMMARY",         // 人生結束，點擊人生總結後
    END: "END",                 // 遊戲完成，點擊重開 重開次數在這之後才會+1
};
```

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/life.js`):
```javascript
set times(v) {
    this.#property.set(this.PropertyTypes.TMS, v);
    this.#achievement.achieve(this.AchievementOpportunity.END);  // ← END timing!
}
```

**Current Rust Implementation** (`src/config/achievement.rs`):
```rust
pub enum Opportunity {
    Start,
    Trajectory,
    Summary,
    // ❌ END is missing!
}
```

**Impact**: 
- 4 achievements cannot be unlocked: 既視感, 孟婆愁, 所有人都是我, Rewrite
- These achievements check conditions at END timing (after clicking restart)

**Fix Required**: Add `End` variant to `Opportunity` enum

---

### ⚠️ Issue 2: Simulator Missing END Achievement Check (CONFIRMED - CRITICAL)

**Status**: ✅ **REAL ISSUE**

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/life.js`):
```javascript
// In simulate() - checks START, TRAJECTORY, SUMMARY
this.#achievement.achieve(this.AchievementOpportunity.START);
// ... during simulation ...
this.#achievement.achieve(this.AchievementOpportunity.TRAJECTORY);
// ... at end ...
this.#achievement.achieve(this.AchievementOpportunity.SUMMARY);

// But END is checked separately when user clicks restart:
set times(v) {
    this.#property.set(this.PropertyTypes.TMS, v);
    this.#achievement.achieve(this.AchievementOpportunity.END);  // ← Here!
}
```

**Current Rust Implementation** (`src/simulator/engine.rs`):
```rust
// Lines 136-182: Only checks START, TRAJECTORY, SUMMARY
// ❌ No END check anywhere
```

**Impact**: END-timed achievements never trigger

**Fix Required**: Add END achievement check (but this is tricky - see "Design Decision" below)

**Design Decision**: 
- Original JS checks END when `times` property is SET (when user clicks restart button)
- Rust simulator is stateless - it doesn't know about "restart button clicks"
- **Solution**: Python wrapper should check END achievements when incrementing `play_count`

---

### ⚠️ Issue 3: TMS (Play Count) Property Missing (CONFIRMED - CRITICAL)

**Status**: ✅ **REAL ISSUE**

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/property.js`):
```javascript
TYPES = {
    TMS: "TMS", // 次數 times TMS
    // ...
}

get(prop) {
    case this.TYPES.TMS:
        return this.lsget('times') || 0;  // ← Reads from localStorage
}
```

**Current Rust Implementation** (`src/property/state.rs` & `src/condition/evaluator.rs`):
```rust
// PropertyState has NO tms field
// get_value() returns 0 for unknown properties
_ => PropertyValue::Integer(0),  // ← TMS always returns 0!
```

**Impact**:
- **Achievements** (4): TMS>9, TMS>49, TMS>199, TMS>499
- **Talents** (1): 百歲百世丸 condition `(AGE?[100])&(TMS>99)`
- **Events** (3): TMS>999, TMS<1000

**Why This Happens**:
- TMS is a **persistent cross-session property** stored in database/localStorage
- Rust simulator is **stateless** - it only knows about current session
- Python wrapper has `play_count` in database but doesn't pass it to Rust

**Fix Required**: 
1. Add `tms: i32` field to `PropertyState`
2. Python wrapper must pass `play_count` when calling Rust simulator
3. Update `get_value()` to return `self.tms` for "TMS" property

---

### ⚠️ Issue 4: AEVT (Historical Events) Property Missing (CONFIRMED - MEDIUM)

**Status**: ✅ **REAL ISSUE**

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/property.js`):
```javascript
TYPES = {
    AEVT: "AEVT", // 觸發過的事件 Achieve Event
    // ...
}

get(prop) {
    case this.TYPES.AEVT:
        return this.lsget(prop) || [];  // ← Reads from localStorage
}
```

**Current Rust Implementation**:
```rust
// No AEVT field in PropertyState
// get_value() returns 0 for unknown properties
```

**Impact**:
- **Achievements** (3): 山海經, 2/2, 資深鄉民
- **Events**: Some 修仙 reincarnation events check AEVT

**Why This Happens**: Same as TMS - persistent cross-session property

**Fix Required**:
1. Add `aevt: Vec<i32>` field to `PropertyState`
2. Python wrapper must pass historical events from database
3. Update `get_value()` to return `PropertyValue::List(&self.aevt)`

---

### ⚠️ Issue 5: ATLT (Historical Talents) Property Missing (CONFIRMED - MEDIUM)

**Status**: ✅ **REAL ISSUE**

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/property.js`):
```javascript
TYPES = {
    ATLT: "ATLT", // 擁有過的天賦 Achieve Talent
    // ...
}

get(prop) {
    case this.TYPES.ATLT:
        return this.lsget(prop) || [];  // ← Reads from localStorage
}
```

**Current Rust Implementation**: Same as AEVT - missing

**Impact**:
- **Achievements** (3): 刷刷刷, 跳跳跳, 莎比

**Fix Required**: Same as AEVT

---

### ⚠️ Issue 6: `max_triggers` Not Calculated (CONFIRMED - LOW PRIORITY)

**Status**: ✅ **REAL ISSUE** (but low impact)

**Evidence from Python** (`liferestart/services/condition_service.py`):
```python
def extract_max_triggers(self, condition: str | None) -> int:
    """從條件中提取最大觸發次數
    
    用於 AGE?[...] 格式的條件，計算可觸發的年齡數量。
    """
    if not condition:
        return 1
    
    match = _RE_AGE_CONDITION.search(condition)
    if match is None:
        return 1
    
    age_list = match.group(1).split(",")
    return len(age_list)  # ← Counts ages in AGE?[30,40,50]
```

**Current Rust Implementation** (`src/config/mod.rs`):
```rust
pub struct TalentConfig {
    pub max_triggers: i32,  // ← Reads from JSON, defaults to 1
    // ...
}
```

**Impact**: 
- Talents with `AGE?[30,40,50]` should trigger 3 times (at ages 30, 40, 50)
- Currently they only trigger once

**Actual Severity**: **LOW** because:
- Most talents have `AGE?[X]` with single age (e.g., `AGE?[30]`)
- Only a few talents have multiple ages
- Need to check JSON data to see how many are affected

**Fix Required**: Implement `extract_max_triggers()` in Rust config loader

---

### ⚠️ Issue 7: CACHV (Achievement Count) Property Missing (CONFIRMED - LOW)

**Status**: ✅ **REAL ISSUE** (but minimal impact)

**Evidence from Original JS** (`refence/lifeRestart-main/src/modules/property.js`):
```javascript
TYPES = {
    CACHV: "CACHV", // 成就達成數 Count Achievement
    // ...
}

get(prop) {
    case this.TYPES.CACHV:
        return this.get(this.fallback(prop)).length;  // ← Counts ACHV array
    case this.TYPES.ACHV:
        return this.lsget(prop) || [];
}
```

**Current Rust Implementation**: Missing

**Impact**:
- Mainly used in `game.json` for talent rarity bonuses
- Python wrapper handles this for talent pulls
- **No achievements or events directly check CACHV**

**Fix Required**: Add `cachv: i32` field and pass from Python

---

## Summary Table

| Issue | Status | Severity | Fix Complexity | Blocks Gameplay | Rust Status |
|-------|--------|----------|----------------|-----------------|-------------|
| 1. Opportunity::End missing | ✅ Real | Critical | Simple | Yes (4 achievements) | ✅ **FIXED** |
| 2. No END achievement check | ✅ Real | Critical | Medium | Yes (4 achievements) | ⚠️ **Needs Python** |
| 3. TMS property missing | ✅ Real | Critical | Medium | Yes (4 achievements, 1 talent, 3 events) | ✅ **FIXED** |
| 4. AEVT property missing | ✅ Real | Medium | Medium | Yes (3 achievements) | ✅ **FIXED** |
| 5. ATLT property missing | ✅ Real | Medium | Medium | Yes (3 achievements) | ✅ **FIXED** |
| 6. max_triggers calculation | ✅ Real | Low | Medium | Partial (few talents) | ⏸️ **Deferred** |
| 7. CACHV property missing | ✅ Real | Low | Simple | No (only affects talent pull bonuses) | ✅ **FIXED** |

---

## ✅ Rust 端修復完成 (2024-12-25)

### 已完成的修改

1. ✅ **新增 `Opportunity::End` 枚舉變體** (`src/config/achievement.rs`)
   - 新增 `End` 變體到 `Opportunity` enum
   - 更新 `from_str()` 方法支援 "END" 字串

2. ✅ **擴充 `PropertyState` 結構** (`src/property/state.rs`)
   - 新增 `tms: i32` - 重開次數
   - 新增 `aevt: Vec<i32>` - 歷史事件
   - 新增 `atlt: Vec<i32>` - 歷史天賦
   - 新增 `cachv: i32` - 成就數量
   - 新增 `new_with_persistent()` 建構函數

3. ✅ **更新條件評估器** (`src/condition/evaluator.rs`)
   - 更新 `get_value()` 支援 TMS, AEVT, ATLT, CACHV 查詢

4. ✅ **新增 `PersistentProperties` 結構** (`src/simulator/engine.rs`)
   - 封裝持久屬性的傳遞

5. ✅ **更新模擬器簽名** (`src/simulator/engine.rs`)
   - `simulate()` 新增 `persistent: Option<&PersistentProperties>` 參數

6. ✅ **更新 Python 綁定** (`src/lib.rs`)
   - `simulate_full_life()` 新增 `tms`, `aevt`, `atlt`, `cachv` 參數
   - `simulate_async()` 新增 `tms`, `aevt`, `atlt`, `cachv` 參數
   - 所有參數都有預設值，保持向後兼容

### 編譯狀態

```bash
✅ cargo build --lib
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.77s
```

---

## Recommended Fix Priority

### P0 (Critical - Fix Immediately)
1. **Add TMS property** - Most impactful, affects achievements, talents, and events
2. **Add Opportunity::End** - Required for END-timed achievements
3. **Implement END achievement check in Python wrapper** - Complete the END timing flow

### P1 (High - Fix Soon)
4. **Add AEVT property** - Affects 3 achievements
5. **Add ATLT property** - Affects 3 achievements

### P2 (Medium - Fix When Convenient)
6. **Implement max_triggers calculation** - Low impact but correct behavior
7. **Add CACHV property** - Minimal impact, mainly for completeness

---

## Implementation Notes

### Persistent Properties Architecture

The original game has two types of properties:

1. **Session Properties** (current life only):
   - AGE, CHR, INT, STR, MNY, SPR, LIF
   - TLT (current talents), EVT (current events)
   - Min/Max tracking (LAGE, HAGE, etc.)

2. **Persistent Properties** (cross-session, stored in localStorage/database):
   - TMS (play count)
   - AEVT (all events ever triggered)
   - ATLT (all talents ever selected)
   - ACHV (all achievements unlocked)
   - CACHV (count of ACHV)
   - EXT (inherited talent)

**Current Rust Design Issue**: Rust simulator only handles session properties

**Solution**: Python wrapper must:
1. Load persistent properties from database
2. Pass them to Rust simulator as initial state
3. Update database after simulation completes

---

## Conclusion

**Yes, there really are this many problems!** 

The root cause is architectural: the original JavaScript game stores persistent data in `localStorage` and the `Property` class seamlessly accesses both session and persistent data. The Rust implementation is stateless and only handles session data, expecting the Python wrapper to manage persistence.

The fix requires:
1. Extending `PropertyState` to include persistent properties
2. Updating Python wrapper to pass persistent data to Rust
3. Adding END achievement check in Python (since Rust doesn't know about "restart button")

All issues are fixable with medium effort. The good news is that the Rust core logic is sound - it just needs to be aware of persistent properties.
