#[derive(Debug, Clone)]
pub enum TimeFrame {
    LastDays(u32),
    LastHours(u32),
}

#[derive(Debug, Clone)]
pub enum GroupBy {
    Hours(u32),
    Minutes(u32),
}
