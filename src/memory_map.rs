use std::{collections::BTreeMap, fmt::Display};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Numeric {
    Number(usize),
    Kb(usize),
}

impl Display for Numeric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Numeric::Number(num) => write!(f, "{}", num),
            Numeric::Kb(num) => write!(f, "{} kB", num),
        }
    }
}

impl Numeric {
    pub fn value(&self) -> usize {
        match self {
            Numeric::Number(num) => *num,
            Numeric::Kb(num) => *num,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Range {
    pub from: usize,
    pub to: usize,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}-{:x}", self.from, self.to)
    }
}

/// Only compare the from
impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.from.partial_cmp(&other.from)
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.from.cmp(&other.from)
    }
}

impl TryFrom<&str> for Range {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split("-").collect::<Vec<_>>();
        Ok(Self {
            from: usize::from_str_radix(split[0], 16).map_err(|_| "cant parse")?,
            to: usize::from_str_radix(split[1], 16).map_err(|_| "cant parse")?,
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MemoryMap {
    pub address_range: Range,
    pub permissions: String,
    pub offset: String,
    pub device: String,
    pub inode: String,
    pub path: Option<String>,
    pub sizes: BTreeMap<String, Numeric>,
    pub vm_flags: String,
}

impl Display for MemoryMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {} {} {} {} {}",
            self.address_range,
            self.permissions,
            self.offset,
            self.device,
            self.inode,
            self.vm_flags,
            self.path.as_ref().map(|s| s.as_str()).unwrap_or_default()
        )?;

        for (key, val) in &self.sizes {
            if val.value() != 0 {
                writeln!(f, "{}={}", key, val)?;
            }
        }

        Ok(())
    }
}

impl PartialOrd for MemoryMap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.address_range.partial_cmp(&other.address_range)
    }
}

impl Ord for MemoryMap {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.address_range.cmp(&other.address_range)
    }
}

impl MemoryMap {
    pub fn size(&self) -> Option<usize> {
        self.sizes.get("Size").map(|size| size.value())
    }

    pub fn rss(&self) -> Option<usize> {
        self.sizes.get("Rss").map(|rss| rss.value())
    }
}
