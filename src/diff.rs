use crate::memory_map::MemoryMap;

#[derive(Default)]
pub struct Diffs {
    pub added: Vec<MemoryMap>,
    pub removed: Vec<MemoryMap>,
    pub changed: Vec<(MemoryMap, MemoryMap)>,
}

pub fn diff_sorted(vec1: &[MemoryMap], vec2: &[MemoryMap]) -> Diffs {
    let mut diffs = Diffs::default();
    let mut i = 0;
    let mut j = 0;

    while i < vec1.len() && j < vec2.len() {
        if vec1[i] < vec2[j] {
            diffs.removed.push(vec1[i].clone());
            i += 1;
        } else if vec1[i] > vec2[j] {
            diffs.added.push(vec2[j].clone());
            j += 1;
        } else {
            if i < vec1.len()
                && j < vec2.len()
                && (vec1[i].address_range.to != vec2[j].address_range.to
                    || vec1[i].size() != vec2[j].size()
                    || vec1[i].rss() != vec2[j].rss())
            {
                diffs.changed.push((vec1[i].clone(), vec2[j].clone()))
            }

            i += 1;
            j += 1;
        }
    }

    // Add remaining elements
    while i < vec1.len() {
        diffs.removed.push(vec1[i].clone());
        i += 1;
    }

    while j < vec2.len() {
        diffs.added.push(vec2[j].clone());
        j += 1;
    }

    diffs
}
