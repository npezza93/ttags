mod café {
    pub struct Widget;
    pub enum Choice { One }
    pub union Value { number: u32 }
    pub type Alias = Widget;
    pub trait Render { fn render(&self); }
    pub const LIMIT: usize = 3;
    pub fn goûter() {}
    impl Widget { pub fn value(&self) {} }
    macro_rules! make { () => {} }
}
