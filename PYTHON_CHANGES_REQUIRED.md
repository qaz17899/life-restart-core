# Python 端修改指南

## ✅ Rust 端已完成的修改

1. ✅ 新增 `Opportunity::End` 枚舉變體
2. ✅ 新增 `PropertyState` 的持久屬性欄位（tms, aevt, atlt, cachv）
3. ✅ 更新 `get_value()` 支援持久屬性查詢
4. ✅ 新增 `PersistentProperties` 結構
5. ✅ 更新 `simulate()` 接受持久屬性參數
6. ✅ 更新 Python 綁定函數簽名

---

## 🔧 Python 端需要修改的地方

### 1. 更新 Rust 模擬器調用（關鍵修改）

**位置**: 呼叫 `life_restart_core.simulate_full_life()` 或 `life_restart_core.simulate_async()` 的地方

**修改前**:
```python
from life_restart_core import simulate_async

result = await simulate_async(
    talent_ids=[1001, 1002, 1003],
    properties={"CHR": 5, "INT": 5, "STR": 5, "MNY": 5},
    achieved_ids={1, 2, 3}
)
```

**修改後**:
```python
from life_restart_core import simulate_async

# 從資料庫讀取持久屬性
player = await get_or_create_player(user_id)
player_stats = await get_player_stats(user_id)

# 讀取歷史事件和天賦
aevt = await LifeRestartTriggeredEvent.filter(user_id=user_id).values_list("event_id", flat=True)
atlt = await LifeRestartCollectedTalent.filter(user_id=user_id).values_list("talent_id", flat=True)

result = await simulate_async(
    talent_ids=[1001, 1002, 1003],
    properties={"CHR": 5, "INT": 5, "STR": 5, "MNY": 5},
    achieved_ids={1, 2, 3},
    # 新增：傳入持久屬性
    tms=player.play_count,
    aevt=list(aevt),
    atlt=list(atlt),
    cachv=player_stats.achievement_count
)
```

---

### 2. 新增 END 成就檢查（關鍵修改）

**位置**: `liferestart/services/player_service.py` 的 `update_statistics()` 方法

**原因**: Rust 模擬器是無狀態的，不知道「重開按鈕」的存在。END 時機的成就必須在 Python 端檢查。

**修改方案**:

```python
async def update_statistics(
    self,
    user_id: int,
    talents: list[int],
    events: list[int],
    new_achievements: list[int],
) -> None:
    """更新玩家統計資料"""
    player = await self.get_or_create_player(user_id)

    # ... 現有的批量插入邏輯 ...

    # 增加重開次數
    player.play_count += 1

    # ✨ 新增：檢查 END 時機成就
    end_achievements = await self._check_end_achievements(user_id, player.play_count)
    if end_achievements:
        # 將 END 成就加入到 new_achievements
        new_achievements.extend(end_achievements)
        
        # 批量插入 END 成就
        now = datetime.now(UTC)
        to_create = [
            LifeRestartAchievement(
                user_id=user_id, 
                achievement_id=aid, 
                achieved_at=now
            )
            for aid in end_achievements
        ]
        if to_create:
            await LifeRestartAchievement.bulk_create(to_create, ignore_conflicts=True)

    # ... 現有的快速模式檢查和保存邏輯 ...

async def _check_end_achievements(self, user_id: int, new_play_count: int) -> list[int]:
    """檢查 END 時機成就
    
    END 時機：遊戲完成，點擊重開後（重開次數在這之後才會+1）
    
    Args:
        user_id: 用戶 ID
        new_play_count: 新的重開次數（已經 +1）
        
    Returns:
        新達成的 END 成就 ID 列表
    """
    from liferestart.services.config_service import get_config_service
    from liferestart.services.condition_service import get_condition_service
    
    config_service = get_config_service()
    condition_service = get_condition_service()
    
    # 取得所有 END 時機的成就
    all_achievements = config_service.get_all_achievements()
    end_achievements = [a for a in all_achievements if a.opportunity == "END"]
    
    # 取得已達成的成就
    achieved_ids = await LifeRestartAchievement.filter(user_id=user_id).values_list(
        "achievement_id", flat=True
    )
    achieved_set = set(achieved_ids)
    
    # 建立屬性存取器（用於條件檢查）
    class PropertyAccessor:
        def __init__(self, tms: int):
            self.tms = tms
            
        def get(self, prop: str):
            if prop == "TMS":
                return self.tms
            # END 成就通常只檢查 TMS
            return 0
    
    accessor = PropertyAccessor(tms=new_play_count)
    
    # 檢查每個 END 成就
    new_end_achievements = []
    for achievement in end_achievements:
        # 跳過已達成的
        if achievement.id in achieved_set:
            continue
            
        # 檢查條件
        if condition_service.check_condition(accessor, achievement.condition):
            new_end_achievements.append(achievement.id)
    
    return new_end_achievements
```

---

### 3. 更新成就檢查邏輯（可選優化）

**位置**: 如果有獨立的成就檢查服務

**說明**: 確保成就檢查時能正確讀取持久屬性。不過由於現在 Rust 端已經處理 START/TRAJECTORY/SUMMARY 的成就檢查，這部分可能不需要修改。

---

## 📋 修改檢查清單

### 必須修改（P0）

- [ ] **找到呼叫 `simulate_async()` 或 `simulate_full_life()` 的地方**
  - 可能在 Cog 或 Service 中
  - 搜尋關鍵字：`simulate_async`, `simulate_full_life`, `life_restart_core`

- [ ] **修改調用處，傳入持久屬性**
  - 從資料庫讀取 `play_count` (TMS)
  - 從資料庫讀取 `aevt` (歷史事件列表)
  - 從資料庫讀取 `atlt` (歷史天賦列表)
  - 從資料庫讀取 `cachv` (成就數量)

- [ ] **在 `player_service.py` 新增 `_check_end_achievements()` 方法**

- [ ] **在 `update_statistics()` 中調用 END 成就檢查**

### 建議修改（P1）

- [ ] **更新類型提示**
  - 如果有 `.pyi` stub 文件，更新函數簽名

- [ ] **更新文檔**
  - 更新 docstring 說明新參數

- [ ] **新增日誌**
  - 記錄持久屬性的傳遞
  - 記錄 END 成就的檢查結果

---

## 🔍 如何找到需要修改的文件

### 方法 1: 搜尋 Rust 模擬器調用

```bash
# 在 liferestart/ 目錄下搜尋
grep -r "simulate_async\|simulate_full_life" liferestart/
```

### 方法 2: 搜尋 life_restart_core 導入

```bash
grep -r "from life_restart_core import\|import life_restart_core" liferestart/
```

### 方法 3: 檢查 Cog 和 Service

可能的位置：
- `liferestart/cogs/*.py` - Discord 指令處理
- `liferestart/services/*.py` - 業務邏輯
- `liferestart/views/*.py` - UI 互動

---

## 🧪 測試建議

### 1. 測試持久屬性傳遞

```python
# 測試 TMS 條件
# 選擇一個需要 TMS>9 的成就或天賦
# 設定 play_count = 10
# 驗證條件能正確觸發

# 測試 AEVT 條件
# 選擇一個需要特定歷史事件的成就
# 在資料庫中插入該事件
# 驗證條件能正確觸發
```

### 2. 測試 END 成就

```python
# 測試「既視感」成就（TMS>9）
# 1. 玩 10 次遊戲
# 2. 第 10 次結束後，檢查是否解鎖「既視感」
# 3. 驗證 play_count 正確增加到 10
```

### 3. 測試向後兼容

```python
# 測試不傳持久屬性（使用預設值）
result = await simulate_async(
    talent_ids=[1001, 1002, 1003],
    properties={"CHR": 5, "INT": 5, "STR": 5, "MNY": 5},
    achieved_ids={1, 2, 3}
    # 不傳 tms, aevt, atlt, cachv
)
# 應該能正常運行，持久屬性使用預設值 0/[]
```

---

## 📊 影響範圍總結

| 修改項目 | 影響範圍 | 優先級 |
|---------|---------|--------|
| 更新 simulate 調用 | 1-2 個文件 | P0 |
| 新增 END 成就檢查 | player_service.py | P0 |
| 更新類型提示 | .pyi 文件 | P1 |
| 更新文檔 | docstring | P1 |

---

## ❓ 常見問題

### Q1: 為什麼 END 成就不在 Rust 端檢查？

**A**: 因為 Rust 模擬器是無狀態的，它只負責模擬「一局人生」。END 時機發生在「點擊重開按鈕」時，這是 UI 層的概念，Rust 不知道。所以 END 成就必須在 Python 端（UI 層）檢查。

### Q2: 如果不傳持久屬性會怎樣？

**A**: 不會報錯，但持久屬性會使用預設值（tms=0, aevt=[], atlt=[], cachv=0）。這意味著：
- 所有需要 TMS>0 的成就/天賦/事件都無法觸發
- 所有需要歷史事件/天賦的成就都無法觸發
- END 成就永遠無法解鎖

### Q3: 需要重新編譯 Python 包嗎？

**A**: 是的！Rust 端修改後需要重新編譯：

```bash
# 開發模式（快速）
maturin develop

# 發布模式（優化）
maturin develop --release
```

### Q4: 如何驗證修改成功？

**A**: 
1. 檢查編譯無錯誤
2. 在 Python 中測試新參數：
   ```python
   from life_restart_core import simulate_async
   help(simulate_async)  # 查看新參數
   ```
3. 運行實際遊戲，檢查 TMS 相關成就能否解鎖

---

## 🎯 下一步

1. **找到調用 Rust 模擬器的地方**（最重要）
2. **修改調用處，傳入持久屬性**
3. **新增 END 成就檢查邏輯**
4. **重新編譯 Python 包**
5. **測試驗證**

需要我幫你找到具體的調用位置嗎？
