//! Property state structure and operations - Optimized version

use rand::seq::SliceRandom;

/// Random property choices for RDM effect
const RDM_PROPERTIES: [&str; 5] = ["CHR", "INT", "STR", "MNY", "SPR"];

/// Property state for a game session
#[derive(Debug, Clone, Default)]
pub struct PropertyState {
    // Basic properties
    pub age: i32,
    pub chr: i32,  // 顏值 (Charisma)
    pub int: i32,  // 智力 (Intelligence)
    pub str_: i32, // 體質 (Strength)
    pub mny: i32,  // 家境 (Money)
    pub spr: i32,  // 快樂 (Spirit)
    pub lif: i32,  // 生命 (Life)

    // List properties - pre-allocated with reasonable capacity
    pub tlt: Vec<i32>, // 天賦列表 (Talents) - typically 3-10 items
    pub evt: Vec<i32>, // 事件列表 (Events) - typically 50-200 items

    // Minimum value tracking
    pub lage: i32,
    pub lchr: i32,
    pub lint: i32,
    pub lstr: i32,
    pub lmny: i32,
    pub lspr: i32,

    // Maximum value tracking
    pub hage: i32,
    pub hchr: i32,
    pub hint: i32,
    pub hstr: i32,
    pub hmny: i32,
    pub hspr: i32,
}

impl PropertyState {
    /// Create a new PropertyState with initial values
    #[inline]
    pub fn new(chr: i32, int: i32, str_: i32, mny: i32, spr: i32, lif: i32) -> Self {
        let mut state = Self {
            age: -1,
            chr,
            int,
            str_,
            mny,
            spr,
            lif,
            // Pre-allocate with typical capacities to avoid reallocations
            tlt: Vec::with_capacity(10),
            evt: Vec::with_capacity(128),
            // Initialize min/max to extreme values
            lage: i32::MAX,
            lchr: i32::MAX,
            lint: i32::MAX,
            lstr: i32::MAX,
            lmny: i32::MAX,
            lspr: i32::MAX,
            hage: i32::MIN,
            hchr: i32::MIN,
            hint: i32::MIN,
            hstr: i32::MIN,
            hmny: i32::MIN,
            hspr: i32::MIN,
        };

        // Initialize min/max with current values
        state.init_min_max();
        state
    }

    /// Initialize min/max tracking with current values
    #[inline]
    fn init_min_max(&mut self) {
        self.lage = self.age;
        self.lchr = self.chr;
        self.lint = self.int;
        self.lstr = self.str_;
        self.lmny = self.mny;
        self.lspr = self.spr;

        self.hage = self.age;
        self.hchr = self.chr;
        self.hint = self.int;
        self.hstr = self.str_;
        self.hmny = self.mny;
        self.hspr = self.spr;
    }

    /// Change a property value by delta - optimized with byte comparison
    #[inline]
    pub fn change(&mut self, prop: &str, delta: i32) {
        // Use byte comparison for faster matching
        match prop.as_bytes() {
            b"AGE" => {
                self.age += delta;
                self.update_age_min_max();
            }
            b"CHR" => {
                self.chr += delta;
                self.update_chr_min_max();
            }
            b"INT" => {
                self.int += delta;
                self.update_int_min_max();
            }
            b"STR" => {
                self.str_ += delta;
                self.update_str_min_max();
            }
            b"MNY" => {
                self.mny += delta;
                self.update_mny_min_max();
            }
            b"SPR" => {
                self.spr += delta;
                self.update_spr_min_max();
            }
            b"LIF" => {
                self.lif += delta;
            }
            b"TLT" => {
                // Linear search is fine for small lists (typically < 10 items)
                if !self.tlt.contains(&delta) {
                    self.tlt.push(delta);
                }
            }
            b"EVT" => {
                // Linear search - could be optimized with HashSet for large lists
                if !self.evt.contains(&delta) {
                    self.evt.push(delta);
                }
            }
            b"RDM" => {
                // Random property
                let mut rng = rand::thread_rng();
                if let Some(random_prop) = RDM_PROPERTIES.choose(&mut rng) {
                    self.change(random_prop, delta);
                }
            }
            _ => {}
        }
    }

    /// Specialized min/max update functions - inlined for performance
    #[inline(always)]
    fn update_age_min_max(&mut self) {
        self.lage = self.lage.min(self.age);
        self.hage = self.hage.max(self.age);
    }

    #[inline(always)]
    fn update_chr_min_max(&mut self) {
        self.lchr = self.lchr.min(self.chr);
        self.hchr = self.hchr.max(self.chr);
    }

    #[inline(always)]
    fn update_int_min_max(&mut self) {
        self.lint = self.lint.min(self.int);
        self.hint = self.hint.max(self.int);
    }

    #[inline(always)]
    fn update_str_min_max(&mut self) {
        self.lstr = self.lstr.min(self.str_);
        self.hstr = self.hstr.max(self.str_);
    }

    #[inline(always)]
    fn update_mny_min_max(&mut self) {
        self.lmny = self.lmny.min(self.mny);
        self.hmny = self.hmny.max(self.mny);
    }

    #[inline(always)]
    fn update_spr_min_max(&mut self) {
        self.lspr = self.lspr.min(self.spr);
        self.hspr = self.hspr.max(self.spr);
    }

    /// Check if the game has ended (LIF < 1)
    #[inline]
    pub fn is_end(&self) -> bool {
        self.lif < 1
    }

    /// Calculate summary score
    /// Formula: (HCHR + HINT + HSTR + HMNY + HSPR) * 2 + HAGE / 2
    #[inline]
    pub fn calculate_summary_score(&self) -> i32 {
        let hchr = self.hchr.max(self.chr);
        let hint = self.hint.max(self.int);
        let hstr = self.hstr.max(self.str_);
        let hmny = self.hmny.max(self.mny);
        let hspr = self.hspr.max(self.spr);
        let hage = self.hage.max(self.age);

        (hchr + hint + hstr + hmny + hspr) * 2 + hage / 2
    }

    /// Get current properties as a HashMap
    #[inline]
    pub fn get_properties_dict(&self) -> std::collections::HashMap<String, i32> {
        let mut props = std::collections::HashMap::with_capacity(6);
        props.insert("AGE".to_string(), self.age);
        props.insert("CHR".to_string(), self.chr);
        props.insert("INT".to_string(), self.int);
        props.insert("STR".to_string(), self.str_);
        props.insert("MNY".to_string(), self.mny);
        props.insert("SPR".to_string(), self.spr);
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = PropertyState::new(5, 5, 5, 5, 5, 1);
        assert_eq!(state.chr, 5);
        assert_eq!(state.int, 5);
        assert_eq!(state.str_, 5);
        assert_eq!(state.mny, 5);
        assert_eq!(state.spr, 5);
        assert_eq!(state.lif, 1);
        assert_eq!(state.age, -1);
    }

    #[test]
    fn test_change_property() {
        let mut state = PropertyState::new(5, 5, 5, 5, 5, 1);
        state.change("CHR", 3);
        assert_eq!(state.chr, 8);
        assert_eq!(state.hchr, 8);
    }

    #[test]
    fn test_min_max_tracking() {
        let mut state = PropertyState::new(5, 5, 5, 5, 5, 1);

        // Increase
        state.change("CHR", 5);
        assert_eq!(state.chr, 10);
        assert_eq!(state.hchr, 10);
        assert_eq!(state.lchr, 5);

        // Decrease
        state.change("CHR", -8);
        assert_eq!(state.chr, 2);
        assert_eq!(state.hchr, 10);
        assert_eq!(state.lchr, 2);
    }

    #[test]
    fn test_is_end() {
        let mut state = PropertyState::new(5, 5, 5, 5, 5, 1);
        assert!(!state.is_end());

        state.change("LIF", -1);
        assert!(state.is_end());
    }

    #[test]
    fn test_summary_score() {
        let mut state = PropertyState::new(10, 10, 10, 10, 10, 1);
        state.age = 100;
        state.hage = 100;

        // (10 + 10 + 10 + 10 + 10) * 2 + 100 / 2 = 100 + 50 = 150
        assert_eq!(state.calculate_summary_score(), 150);
    }

    #[test]
    fn test_talent_list() {
        let mut state = PropertyState::new(5, 5, 5, 5, 5, 1);
        state.change("TLT", 1001);
        state.change("TLT", 1002);
        state.change("TLT", 1001); // Duplicate, should not add

        assert_eq!(state.tlt.len(), 2);
        assert!(state.tlt.contains(&1001));
        assert!(state.tlt.contains(&1002));
    }
}
